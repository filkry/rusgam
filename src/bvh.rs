use allocate::{STACK_ALLOCATOR, SMemQueue, SMemVec};
use collections::{SPoolHandle, SPool};
use safewindows;
use utils::{SAABB};

#[derive(Clone)]
struct SLeafNode {
    bounds: SAABB,
    parent: SPoolHandle,
    owner: SPoolHandle,
}

#[derive(Clone)]
struct SInternalNode {
    bounds: SAABB,
    parent: SPoolHandle,
    child1: SPoolHandle,
    child2: SPoolHandle,
}

#[derive(Clone)]
enum ENode {
    Free,
    Leaf(SLeafNode),
    Internal(SInternalNode),
}

pub struct STree {
    nodes: SPool<ENode>,
    root: SPoolHandle,
}

impl ENode {
    pub fn parent(&self) -> SPoolHandle {
        match self {
            Self::Free => {
                break_assert!(false);
                SPoolHandle::default()
            },
            Self::Leaf(leaf) => leaf.parent,
            Self::Internal(internal) => internal.parent,
        }
    }

    pub fn bounds(&self) -> Option<&SAABB> {
        match self {
            Self::Free => {
                break_assert!(false);
                None
            },
            Self::Leaf(leaf) => Some(&leaf.bounds),
            Self::Internal(internal) => Some(&internal.bounds),
        }
    }

    pub fn clear_parent(&mut self) {
        match self {
            Self::Free => {
                break_assert!(false);
            },
            Self::Leaf(leaf) => { leaf.parent.invalidate() },
            Self::Internal(internal) => { internal.parent.invalidate() },
        }
    }

    pub fn set_parent(&mut self, new_parent: SPoolHandle) {
        match self {
            Self::Free => {
                break_assert!(false);
            },
            Self::Leaf(leaf) => { leaf.parent = new_parent },
            Self::Internal(internal) => { internal.parent = new_parent },
        }
    }

    pub fn set_bounds(&mut self, new_bounds: &SAABB) {
        match self {
            Self::Free => {
                break_assert!(false);
            },
            Self::Leaf(leaf) => { leaf.bounds = new_bounds.clone() },
            Self::Internal(internal) => { internal.bounds = new_bounds.clone() },
        }
    }
}

impl Default for ENode {
    fn default() -> Self {
        Self::Free
    }
}

impl STree {
    fn union(&self, a: SPoolHandle, b: SPoolHandle) -> SAABB {
        let a_aabb = self.nodes.get(a).unwrap().bounds();
        let b_aabb = self.nodes.get(b).unwrap().bounds();
        match (a_aabb, b_aabb) {
            (Some(a_aabb_int), Some(b_aabb_int)) => SAABB::union(a_aabb_int, b_aabb_int),
            (Some(a_aabb_int), None) => a_aabb_int.clone(),
            (None, Some(b_aabb_int)) => b_aabb_int.clone(),
            (None, None) => SAABB::zero(),
        }
    }

    fn find_best_sibling(&self, query_node: SPoolHandle) -> SPoolHandle {
        self.tree_valid();

        struct SSearch {
            node_handle: SPoolHandle,
            inherited_cost: f32,
        }

        STACK_ALLOCATOR.with(|sa| -> SPoolHandle {
            let mut search_queue = SMemQueue::<SSearch>::new(sa, self.nodes.used()).unwrap();
            break_assert!(self.root.valid());
            let mut best = self.root;
            let mut best_cost = self.union(query_node, best).surface_area();
            search_queue.push_back(SSearch{
                node_handle: best,
                inherited_cost: best_cost - self.nodes.get(best).unwrap().bounds().unwrap().surface_area(),
            });

            let query_node_sa = self.nodes.get(query_node).unwrap().bounds().unwrap().surface_area();

            while let Some(cur_search) = search_queue.pop_front() {
                let direct_cost = self.union(query_node, cur_search.node_handle).surface_area();
                let total_cost = direct_cost + cur_search.inherited_cost;
                if total_cost < best_cost {
                    best = cur_search.node_handle;
                    best_cost = total_cost;
                }

                if let ENode::Internal(internal) = self.nodes.get(cur_search.node_handle).unwrap() {
                    let cur_node_sa = internal.bounds.surface_area();
                    let new_inherited_cost = direct_cost - cur_node_sa;
                    let children_inherited_cost = new_inherited_cost + cur_search.inherited_cost;

                    let lower_bound_cost = query_node_sa + children_inherited_cost;

                    if lower_bound_cost < best_cost {
                        if internal.child1.valid() {
                            search_queue.push_back(SSearch{
                                node_handle: internal.child1,
                                inherited_cost: children_inherited_cost,
                            });
                        }
                        if internal.child2.valid() {
                            search_queue.push_back(SSearch{
                                node_handle: internal.child2,
                                inherited_cost: children_inherited_cost,
                            });
                        }
                    }
                }
            }

            break_assert!(best.valid());

            best
        })
    }

    pub fn new() -> Self {
        Self {
            nodes: SPool::create_default(0, 1024),
            root: SPoolHandle::default(),
        }
    }

    fn update_bounds_from_children(&mut self, node_handle: SPoolHandle) {
        let (child1, child2) = {
            if let ENode::Internal(internal) = self.nodes.get(node_handle).unwrap() {
                (internal.child1, internal.child2)
            }
            else {
                break_assert!(false);
                (SPoolHandle::default(), SPoolHandle::default())
            }
        };

        let mut new_bounds = SAABB::zero();
        if child1.valid() {
            new_bounds = SAABB::union(&new_bounds, &self.nodes.get(child1).unwrap().bounds().unwrap());
        }
        if child2.valid() {
            new_bounds = SAABB::union(&new_bounds, &self.nodes.get(child2).unwrap().bounds().unwrap());
        }
        self.nodes.get_mut(node_handle).unwrap().set_bounds(&new_bounds);
    }

    pub fn insert(&mut self, owner: SPoolHandle, bounds: &SAABB) -> SPoolHandle {
        let first : bool = self.nodes.used() == 0;
        let leaf_handle = self.nodes.alloc().unwrap();

        // -- initialize node
        {
            let node = self.nodes.get_mut(leaf_handle).unwrap();
            *node = ENode::Leaf(SLeafNode{
                bounds: bounds.clone(),
                parent: Default::default(),
                owner: owner,
            });
        }

        if first {
            self.root = leaf_handle;
            return leaf_handle;
        }

        // -- Step 1: find the best sibling for the new leaf
        let sibling_handle = self.find_best_sibling(leaf_handle);

        // -- Step 2: create a new parent
        let old_parent_handle = self.nodes.get(sibling_handle).unwrap().parent();

        let new_parent_handle = self.nodes.alloc().unwrap();
        {
            let new_bounds = SAABB::union(
                self.nodes.get(sibling_handle).unwrap().bounds().unwrap(),
                bounds,
            );
            let new_parent = self.nodes.get_mut(new_parent_handle).unwrap();
            *new_parent = ENode::Internal(SInternalNode{
                bounds: new_bounds,
                parent: old_parent_handle,
                child1: sibling_handle,
                child2: leaf_handle,
            });
        }

        self.nodes.get_mut(sibling_handle).unwrap().set_parent(new_parent_handle);
        self.nodes.get_mut(leaf_handle).unwrap().set_parent(new_parent_handle);

        if old_parent_handle.valid() {
            // -- sibling was not the root

            if let ENode::Internal(internal) = self.nodes.get_mut(old_parent_handle).unwrap() {
                // -- update old parent's child to the new parent
                if internal.child1 == sibling_handle {
                    internal.child1 = new_parent_handle;
                }
                else {
                    internal.child2 = new_parent_handle;
                }
            }
            else {
                break_assert!(false)
            }
        }
        else {
            // -- sibling was the root, new parent is now root
            self.root = new_parent_handle;
        }

        // -- Step 3: walk up, refitting AABBs
        let mut cur_handle = self.nodes.get(leaf_handle).unwrap().parent();
        while cur_handle.valid() {
            self.update_bounds_from_children(cur_handle);
            cur_handle = self.nodes.get(cur_handle).unwrap().parent();
        }

        self.tree_valid();

        leaf_handle
    }

    pub fn get_bvh_heirarchy_for_entry(&self, entry: SPoolHandle, output: &mut SMemVec<SAABB>) {
        let mut cur_handle = entry;
        while cur_handle.valid() {
            output.push(self.nodes.get(cur_handle).unwrap().bounds().unwrap().clone());
            cur_handle = self.nodes.get(cur_handle).unwrap().parent();
        }
    }

    fn tree_valid(&self) -> bool {
        STACK_ALLOCATOR.with(|sa| -> bool {
            let mut search_queue = SMemQueue::<SPoolHandle>::new(sa, self.nodes.used()).unwrap();
            let mut child_count = SMemVec::<u16>::new(sa, self.nodes.max() as usize, 0).unwrap();
            for _ in 0..self.nodes.max() {
                child_count.push(0);
            }

            search_queue.push_back(self.root);

            while let Some(cur_handle) = search_queue.pop_front() {
                // -- check handle is valid
                if !self.nodes.get(cur_handle).is_ok() {
                    break_assert!(false);
                    return false;
                }

                // -- check we are a valid node type
                if let ENode::Free = self.nodes.get(cur_handle).unwrap() {
                    break_assert!(false);
                    return false;
                }

                // -- check valid parent
                if !(self.root == cur_handle) {
                    let parent_handle = self.nodes.get(cur_handle).unwrap().parent();

                    // -- parent handle is valid
                    if !parent_handle.valid() {
                        break_assert!(false);
                        return false;
                    }

                    // -- parent handle can be resolved
                    if !self.nodes.get(parent_handle).is_ok() {
                        break_assert!(false);
                        return false;
                    }

                    // -- parent is an internal node, and cur_handle is child of parent
                    let parent = self.nodes.get(parent_handle).unwrap();
                    if let ENode::Internal(internal) = parent {
                        if !(internal.child1 == cur_handle || internal.child2 == cur_handle) {
                            break_assert!(false);
                            return false;
                        }
                    }
                    else {
                        break_assert!(false);
                        return false;
                    }

                    child_count[parent_handle.index() as usize] += 1;
                    if child_count[parent_handle.index() as usize] > 2 {
                        break_assert!(false);
                        return false;
                    }
                }
                else {
                    // -- if we are the root, should invalid parent
                    let parent_handle = self.nodes.get(cur_handle).unwrap().parent();
                    if parent_handle.valid() {
                        break_assert!(false);
                        return false;
                    }
                }

                // -- if internal, check validity
                let node = self.nodes.get(cur_handle).unwrap();
                if let ENode::Internal(internal) = node {

                    // -- both children must be valid
                    if !internal.child1.valid() || !internal.child2.valid() {
                        break_assert!(false);
                        return false;
                    }

                    // -- must have different children
                    if internal.child1 == internal.child2 {
                        break_assert!(false);
                        return false;
                    }

                    // -- children must resolve if valid
                    if !self.nodes.get(internal.child1).is_ok() {
                        break_assert!(false);
                        return false;
                    }
                    if !self.nodes.get(internal.child2).is_ok() {
                        break_assert!(false);
                        return false;
                    }

                    // -- aabb must be tight around child aabbs
                    let child1_aabb = self.nodes.get(internal.child1).unwrap().bounds().unwrap();
                    let child2_aabb = self.nodes.get(internal.child2).unwrap().bounds().unwrap();
                    let unified_aabb = SAABB::union(child1_aabb, child2_aabb);

                    if !(internal.bounds == unified_aabb) {
                        println!("Mismatch:");
                        println!("{:?}", internal.bounds);
                        println!("{:?}", internal.bounds);
                        break_assert!(false);
                        return false;
                    }

                    // -- push children to recursively test
                    search_queue.push_back(internal.child1);
                    search_queue.push_back(internal.child2);
                }
            }

            return true;
        })
    }

    pub fn remove(&mut self, entry: SPoolHandle) {
        let mut handle_to_delete = entry;
        drop(entry);

        while handle_to_delete.valid() {
            let parent_handle = self.nodes.get(handle_to_delete).unwrap().parent();
            let parent_parent_handle = self.nodes.get(parent_handle).unwrap().parent();

            let mut other_child_handle = SPoolHandle::default();
            {
                let parent = self.nodes.get_mut(parent_handle).unwrap(); // $$$FRK(TODO): change to unchecked when more confident
                if let ENode::Internal(int) = parent {
                    if int.child1 == handle_to_delete {
                        int.child1.invalidate();
                        other_child_handle = int.child2;
                    }
                    else {
                        break_assert!(int.child2 == handle_to_delete);
                        int.child2.invalidate();
                        other_child_handle = int.child1;
                    }
                }
                else {
                    break_assert!(false);
                }
            }
            *self.nodes.get_mut(handle_to_delete).unwrap() = ENode::Free;
            self.nodes.free(handle_to_delete);

            if other_child_handle.valid() {
                if parent_parent_handle.valid() {
                    // -- patch other_child in to replace parent in parent_parent
                    let parent_parent = self.nodes.get_mut(parent_parent_handle).unwrap(); // $$$FRK(TODO): change to unchecked when more confident
                    if let ENode::Internal(int) = parent_parent {
                        if int.child1 == parent_handle {
                            int.child1 = other_child_handle;
                        }
                        else {
                            break_assert!(int.child2 == parent_handle);
                            int.child2 = other_child_handle;
                        }
                    }
                    else {
                        break_assert!(false);
                    }
                    *self.nodes.get_mut(parent_handle).unwrap() = ENode::Free;
                    self.nodes.free(parent_handle);

                    self.nodes.get_mut(other_child_handle).unwrap().set_parent(parent_parent_handle);

                    // -- recompute AABBs up the tree
                    let mut recompute_handle = parent_parent_handle;
                    while recompute_handle.valid() {
                        self.update_bounds_from_children(recompute_handle);
                        recompute_handle = self.nodes.get(recompute_handle).unwrap().parent();
                    }
                }
                else {
                    // -- parent was root, now other_child is the root
                    break_assert!(self.root == parent_handle);
                    *self.nodes.get_mut(parent_handle).unwrap() = ENode::Free;
                    self.nodes.free(parent_handle);
                    self.root = other_child_handle;

                    self.nodes.get_mut(other_child_handle).unwrap().clear_parent();
                }

                handle_to_delete.invalidate();
            }
            else {
                // -- recursively delete the parent, since it's an internal node with no children
                if let ENode::Internal(int) = self.nodes.get(parent_handle).unwrap() {
                    break_assert!(int.child1.valid() && int.child2.valid())
                }
                else {
                    break_assert!(false);
                }

                handle_to_delete = parent_handle;
            }
        }

        self.tree_valid();
    }

    pub fn imgui_menu(&self, imgui_ui: &imgui::Ui) {
        use imgui::*;

        STACK_ALLOCATOR.with(|sa| {
            let mut to_show = SMemVec::<SPoolHandle>::new(sa, self.nodes.used() as usize, 0).unwrap();
            to_show.push(self.root);

            imgui_ui.menu(imgui::im_str!("BVH"), true, || {

                while let Some(cur_handle) = to_show.pop() {
                    if imgui_ui.collapsing_header(&im_str!("Node {}.{}", cur_handle.index(), cur_handle.generation())).build() {
                        imgui_ui.indent();
                        let node = self.nodes.get(cur_handle).unwrap();
                        if let ENode::Leaf(leaf) = node {
                            imgui_ui.text(&im_str!("Owner: {}.{}", leaf.owner.index(), leaf.owner.generation()));
                            imgui_ui.unindent();
                        }
                        else if let ENode::Internal(internal) = node {
                            to_show.push(internal.child1);
                            to_show.push(internal.child2);
                        }
                    }
                }
            });
        });
    }
}