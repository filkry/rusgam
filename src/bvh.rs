use allocate::{STACK_ALLOCATOR, SMemQueue, SMemVec};
use collections::{SPoolHandle, SPool};
use safewindows;
use utils::{SAABB, SRay, ray_intersects_aabb};

pub type SNodeHandle = SPoolHandle<u16, u16>;

#[derive(Clone)]
struct SLeafNode<TOwner: Clone> {
    bounds: SAABB,
    parent: SNodeHandle,
    owner: TOwner,
}

#[derive(Clone)]
struct SInternalNode {
    bounds: SAABB,
    parent: SNodeHandle,
    child1: SNodeHandle,
    child2: SNodeHandle,
}

#[derive(Clone)]
enum ENode<TOwner: Clone> {
    Free,
    Leaf(SLeafNode<TOwner>),
    Internal(SInternalNode),
}

pub struct STree<TOwner: Clone> {
    nodes: SPool<ENode<TOwner>, u16, u16>,
    root: SNodeHandle,
}

impl<TOwner: Clone> ENode<TOwner> {
    pub fn parent(&self) -> SNodeHandle {
        match self {
            Self::Free => {
                break_assert!(false);
                SNodeHandle::default()
            },
            Self::Leaf(leaf) => leaf.parent,
            Self::Internal(internal) => internal.parent,
        }
    }

    pub fn bounds(&self) -> &SAABB {
        match self {
            Self::Free => {
                panic!("trying to get bounds of freed node");
            },
            Self::Leaf(leaf) => &leaf.bounds,
            Self::Internal(internal) => &internal.bounds,
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

    pub fn set_parent(&mut self, new_parent: SNodeHandle) {
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

    pub fn owner(&self) -> Result<TOwner, &'static str> {
        if let Self::Leaf(leaf) = self {
            return Ok(leaf.owner.clone());
        }

        Err("asked for owner of non-leaf node!")
    }
}

impl<TOwner: Clone> Default for ENode<TOwner> {
    fn default() -> Self {
        Self::Free
    }
}

impl<TOwner: Clone> STree<TOwner> {
    fn union(&self, a: SNodeHandle, b: SNodeHandle) -> SAABB {
        let a_aabb = self.nodes.get(a).expect("pass valid handles").bounds();
        let b_aabb = self.nodes.get(b).expect("pass valid handles").bounds();
        SAABB::union(a_aabb, b_aabb)
    }

    fn find_best_sibling(&self, query_node: SNodeHandle) -> SNodeHandle {
        self.tree_valid();

        struct SSearch {
            node_handle: SNodeHandle,
            inherited_cost: f32,
        }

        STACK_ALLOCATOR.with(|sa| -> SNodeHandle {
            let mut search_queue = SMemQueue::<SSearch>::new(&sa.as_ref(), self.nodes.used()).expect("blew stack allocator");
            break_assert!(self.root.valid());
            let mut best = self.root;
            let mut best_cost = self.union(query_node, best).surface_area();
            search_queue.push_back(SSearch{
                node_handle: best,
                inherited_cost: best_cost - self.nodes.get(best).expect("best must always be valid").bounds().surface_area(),
            });

            let query_node_sa = self.nodes.get(query_node)
                .expect("query node must always be valid").bounds().surface_area();

            while let Some(cur_search) = search_queue.pop_front() {
                let direct_cost = self.union(query_node, cur_search.node_handle).surface_area();
                let total_cost = direct_cost + cur_search.inherited_cost;
                if total_cost < best_cost {
                    best = cur_search.node_handle;
                    best_cost = total_cost;
                }

                if let ENode::Internal(internal) = self.nodes.get(cur_search.node_handle).expect("search should never hit invalid handle") {
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
            root: SNodeHandle::default(),
        }
    }

    pub fn owner(&self, node_handle: SNodeHandle) -> TOwner {
        self.nodes.get(node_handle).expect("invalid entry").owner().expect("asked for owner of non-leaf!")
    }

    fn update_bounds_from_children(&mut self, node_handle: SNodeHandle) {
        let (child1, child2) = {
            if let ENode::Internal(internal) = self.nodes.get(node_handle).expect("pass valid handles") {
                (internal.child1, internal.child2)
            }
            else {
                break_assert!(false);
                (SNodeHandle::default(), SNodeHandle::default())
            }
        };

        let new_bounds = SAABB::union(
            &self.nodes.get(child1).expect("node has invalid children").bounds(),
            &self.nodes.get(child2).expect("node has invalid children").bounds(),
        );

        self.nodes.get_mut(node_handle).expect("pass valid handles").set_bounds(&new_bounds);
    }

    fn replace_child_without_updating_bounds(&mut self, parent: SNodeHandle, original_child: SNodeHandle, new_child: SNodeHandle) {
        if let ENode::Internal(internal) = self.nodes.get_mut(parent).expect("pass valid handles") {
            if internal.child1 == original_child {
                internal.child1 = new_child;
            }
            else if internal.child2 == original_child {
                internal.child2 = new_child;
            }
            else {
                break_assert!(false);
            }
        }
        else {
            break_assert!(false);
        }
    }

    fn swap_nodes_without_updating_bounds(&mut self, node_a: SNodeHandle, node_b: SNodeHandle) {
        let node_a_original_parent = self.nodes.get(node_a).expect("pass valid handles").parent();
        let node_b_original_parent = self.nodes.get(node_b).expect("pass valid handles").parent();

        self.nodes.get_mut(node_a).expect("expected above").set_parent(node_b_original_parent);
        self.replace_child_without_updating_bounds(node_b_original_parent, node_b, node_a);

        self.nodes.get_mut(node_b).expect("expected above").set_parent(node_a_original_parent);
        self.replace_child_without_updating_bounds(node_a_original_parent, node_a, node_b);
    }

    fn rotate_children_grandchildren(&mut self, node_handle: SNodeHandle) {
        #[derive(Clone, Copy)]
        struct SSwap {
            child: SNodeHandle,
            other_child: SNodeHandle,
            grandchild: SNodeHandle,
            sa_diff: f32,
        }
        let mut best_swap : Option<SSwap> = None;

        if let ENode::Internal(internal) = self.nodes.get(node_handle).expect("pass valid handle") {
            let mut test_grandchild = |
                swap_child : SNodeHandle,
                other_child : SNodeHandle,
                swap_child_cur_sa : f32,
                swap_grandchild : SNodeHandle,
                other_grandchild : SNodeHandle,
            | {
                let possible_bounds = SAABB::union(
                    &self.nodes.get(swap_child).expect("somehow bad handle").bounds(),
                    &self.nodes.get(other_grandchild).expect("somehow bad handle").bounds(),
                );
                let possible_sa = possible_bounds.surface_area();
                let sa_diff = swap_child_cur_sa - possible_sa;
                if (sa_diff > 0.0) && (best_swap.is_none() || best_swap.expect("checked here").sa_diff < sa_diff) {
                    best_swap = Some(SSwap{
                        child: swap_child,
                        other_child: other_child,
                        grandchild: swap_grandchild,
                        sa_diff: sa_diff,
                    });
                }
            };

            let mut test_child = |swap_child : SNodeHandle, other_child: SNodeHandle| {
                let cur_bounds = self.nodes.get(other_child).expect("pass valid handle").bounds();
                let cur_sa = cur_bounds.surface_area();
                if let ENode::Internal(other_child_internal) = self.nodes.get(other_child).expect("pass valid handle") {
                    test_grandchild(swap_child, other_child, cur_sa, other_child_internal.child1, other_child_internal.child2);
                    test_grandchild(swap_child, other_child, cur_sa, other_child_internal.child2, other_child_internal.child1);
                }
            };

            // -- looking at grandchildren under child2
            test_child(internal.child1, internal.child2);
            test_child(internal.child2, internal.child1);
        }

        if let Some(best_swap_int) = &best_swap {
            self.swap_nodes_without_updating_bounds(best_swap_int.child, best_swap_int.grandchild);
            self.update_bounds_from_children(best_swap_int.other_child);
        }
    }

    pub fn insert(&mut self, owner: TOwner, bounds: &SAABB, fixed_handle: Option<SNodeHandle>) -> Result<SNodeHandle, &'static str> {
        let first : bool = self.nodes.used() == 0;
        let leaf_handle = match fixed_handle {
            Some(h) => h,
            None => self.nodes.alloc()?,
        };

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
            return Ok(leaf_handle);
        }

        // -- Step 1: find the best sibling for the new leaf
        let sibling_handle = self.find_best_sibling(leaf_handle);

        // -- Step 2: create a new parent
        let old_parent_handle = self.nodes.get(sibling_handle).unwrap().parent();

        let new_parent_handle = self.nodes.alloc().unwrap();
        {
            let new_bounds = SAABB::union(
                self.nodes.get(sibling_handle).unwrap().bounds(),
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
            self.rotate_children_grandchildren(cur_handle);

            cur_handle = self.nodes.get(cur_handle).unwrap().parent();
        }

        self.tree_valid();

        Ok(leaf_handle)
    }

    pub fn get_bvh_heirarchy_for_entry(&self, entry: SNodeHandle, output: &mut SMemVec<SAABB>) {
        let mut cur_handle = entry;
        while cur_handle.valid() {
            output.push(self.nodes.get(cur_handle).unwrap().bounds().clone());
            cur_handle = self.nodes.get(cur_handle).unwrap().parent();
        }
    }

    fn tree_valid(&self) -> bool {
        STACK_ALLOCATOR.with(|sa| -> bool {
            let mut search_queue = SMemQueue::<SNodeHandle>::new(&sa.as_ref(), self.nodes.used()).unwrap();
            let mut child_count = SMemVec::<u16>::new(&sa.as_ref(), self.nodes.max() as usize, 0).unwrap();
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
                    let child1_aabb = self.nodes.get(internal.child1).unwrap().bounds();
                    let child2_aabb = self.nodes.get(internal.child2).unwrap().bounds();
                    let unified_aabb = SAABB::union(child1_aabb, child2_aabb);

                    if !(internal.bounds == unified_aabb) {
                        println!("Mismatch:");
                        println!("{:?}", internal.bounds);
                        println!("{:?}", unified_aabb);
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

    pub fn update_entry(&mut self, entry: SNodeHandle, bounds: &SAABB) {
        let owner = self.owner(entry);
        self.remove(entry.clone(), false);
        self.insert(owner, bounds, Some(entry)).expect("allocation should never fail since we kept our handle");
    }

    pub fn remove(&mut self, target_entry: SNodeHandle, free_entry: bool) {
        let mut handle_to_delete = target_entry;

        while handle_to_delete.valid() {
            let parent_handle = self.nodes.get(handle_to_delete).expect("produced bad handle").parent();
            let parent_parent_handle = self.nodes.get(parent_handle).expect("produced bad handle").parent();

            let mut other_child_handle = SNodeHandle::default();
            {
                let parent = self.nodes.get_mut(parent_handle).expect("should never have invalid parent");
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
            *self.nodes.get_mut(handle_to_delete).expect("produced bad handle") = ENode::Free;

            if handle_to_delete != target_entry || free_entry {
                self.nodes.free(handle_to_delete);
            }

            if other_child_handle.valid() {
                if parent_parent_handle.valid() {
                    // -- patch other_child in to replace parent in parent_parent
                    let parent_parent = self.nodes.get_mut(parent_parent_handle).expect("should never have invalid parent");
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
                    *self.nodes.get_mut(parent_handle).expect("produced bad handle") = ENode::Free;
                    self.nodes.free(parent_handle);

                    self.nodes.get_mut(other_child_handle).expect("produced bad handle").set_parent(parent_parent_handle);

                    // -- recompute AABBs up the tree
                    let mut recompute_handle = parent_parent_handle;
                    while recompute_handle.valid() {
                        self.update_bounds_from_children(recompute_handle);
                        recompute_handle = self.nodes.get(recompute_handle).expect("produced bad handle").parent();
                    }
                }
                else {
                    // -- parent was root, now other_child is the root
                    break_assert!(self.root == parent_handle);
                    *self.nodes.get_mut(parent_handle).expect("produced bad handle") = ENode::Free;
                    self.nodes.free(parent_handle);
                    self.root = other_child_handle;

                    self.nodes.get_mut(other_child_handle).expect("produced bad handle").clear_parent();
                }

                handle_to_delete.invalidate();
            }
            else {
                // -- recursively delete the parent, since it's an internal node with no children
                if let ENode::Internal(int) = self.nodes.get(parent_handle).expect("produced bad handle") {
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

    pub fn compute_height(&self) -> usize {
        struct SSearchItem {
            node: SNodeHandle,
            height: usize,
        }

        let mut max_height = 0;

        STACK_ALLOCATOR.with(|sa| {
            let mut to_search = SMemVec::<SSearchItem>::new(&sa.as_ref(), self.nodes.used() as usize, 0).unwrap();
            to_search.push(SSearchItem{
                node: self.root,
                height: 1,
            });

            while let Some(search_item) = to_search.pop() {
                let node = self.nodes.get(search_item.node).unwrap();
                max_height = std::cmp::max(max_height, search_item.height);

                if let ENode::Leaf(_) = node {
                    // -- do nothing else
                }
                else if let ENode::Internal(internal) = node {
                    to_search.push(SSearchItem{
                        node: internal.child1,
                        height: search_item.height + 1,
                    });
                    to_search.push(SSearchItem{
                        node: internal.child2,
                        height: search_item.height + 1,
                    });
                }
            }
        });

        return max_height;
    }

    pub fn compute_average_leaf_height(&self) -> f32 {
        struct SSearchItem {
            node: SNodeHandle,
            height: f32,
        }

        let mut total_height = 0.0;
        let mut num_leaves = 0.0;

        STACK_ALLOCATOR.with(|sa| {
            let mut to_search = SMemVec::<SSearchItem>::new(&sa.as_ref(), self.nodes.used() as usize, 0).unwrap();
            to_search.push(SSearchItem{
                node: self.root,
                height: 1.0,
            });

            while let Some(search_item) = to_search.pop() {
                let node = self.nodes.get(search_item.node).unwrap();

                if let ENode::Leaf(_) = node {
                    total_height += search_item.height;
                    num_leaves += 1.0;
                }
                else if let ENode::Internal(internal) = node {
                    to_search.push(SSearchItem{
                        node: internal.child1,
                        height: search_item.height + 1.0,
                    });
                    to_search.push(SSearchItem{
                        node: internal.child2,
                        height: search_item.height + 1.0,
                    });
                }
            }
        });

        if num_leaves > 0.0 {
            return total_height / num_leaves;
        }
        else {
            return 0.0;
        }
    }

    // -- returns all leaf nodes, and the t to their start
    pub fn cast_ray(&self, ray: &SRay, out: &mut SMemVec<(f32, TOwner)>) {
        if self.nodes.used() == 0 {
            return;
        }

        STACK_ALLOCATOR.with(|sa| {
            let mut to_search = SMemVec::<SNodeHandle>::new(&sa.as_ref(), self.nodes.used() as usize, 0).unwrap();
            to_search.push(self.root);

            while let Some(cur_handle) = to_search.pop() {
                let node = self.nodes.get(cur_handle).unwrap();
                let aabb = &node.bounds();

                if let Some(t) = ray_intersects_aabb(&ray, aabb) {
                    if let ENode::Internal(internal) = node {
                        to_search.push(internal.child1);
                        to_search.push(internal.child2);
                    }
                    else if let ENode::Leaf(leaf) = node {
                        out.push((t, leaf.owner.clone()));
                    }
                }
            }
        });
    }

    pub fn imgui_menu(&self, imgui_ui: &imgui::Ui, draw_selected_bvh: &mut bool) {
        use imgui::*;

        if self.nodes.used() == 0 {
            return;
        }

        STACK_ALLOCATOR.with(|sa| {
            let mut to_show = SMemVec::<SNodeHandle>::new(&sa.as_ref(), self.nodes.used() as usize, 0).unwrap();
            to_show.push(self.root);

            imgui_ui.menu(imgui::im_str!("BVH"), true, || {
                imgui_ui.text(&im_str!("Tree height: {}", self.compute_height()));
                imgui_ui.text(&im_str!("Average leaf height: {}", self.compute_average_leaf_height()));
                imgui_ui.checkbox(&im_str!("Draw selected entity's BVH"), draw_selected_bvh);

                while let Some(cur_handle) = to_show.pop() {
                    if imgui_ui.collapsing_header(&im_str!("Node {}.{}", cur_handle.index(), cur_handle.generation())).build() {
                        imgui_ui.indent();
                        let node = self.nodes.get(cur_handle).unwrap();
                        if let ENode::Leaf(_leaf) = node {
                            panic!("re-implement");
                            //imgui_ui.text(&im_str!("Owner: {}.{}", leaf.owner.index(), leaf.owner.generation()));
                            //imgui_ui.unindent();
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

