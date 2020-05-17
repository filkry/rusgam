use allocate::{STACK_ALLOCATOR, SMemQueue};
use collections::{SPoolHandle, SPool};
use safewindows;
use utils::{SAABB};

struct SLeafNode {
    bounds: SAABB,
    parent: SPoolHandle,
    owner: SPoolHandle,
}

struct SInternalNode {
    bounds: SAABB,
    parent: SPoolHandle,
    child1: SPoolHandle,
    child2: SPoolHandle,
}

enum ENode {
    Free,
    Leaf(SLeafNode),
    Internal(SInternalNode),
}

pub struct Tree {
    nodes: SPool<ENode>,
    node_count: u32,
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

impl Tree {
    fn union(&self, a: SPoolHandle, b: SPoolHandle) -> SAABB {
        let a_aabb = self.nodes.get_unchecked(a).bounds();
        let b_aabb = self.nodes.get_unchecked(b).bounds();
        match (a_aabb, b_aabb) {
            (Some(a_aabb_int), Some(b_aabb_int)) => SAABB::union(a_aabb_int, b_aabb_int),
            (Some(a_aabb_int), None) => a_aabb_int.clone(),
            (None, Some(b_aabb_int)) => b_aabb_int.clone(),
            (None, None) => SAABB::default(),
        }
    }

    fn find_best_sibling(&self, query_node: SPoolHandle) -> SPoolHandle {
        struct SSearch {
            node_handle: SPoolHandle,
            inherited_cost: f32,
        }

        STACK_ALLOCATOR.with(|sa| -> SPoolHandle {
            let mut search_queue = SMemQueue::<SSearch>::new(sa, self.nodes.used()).unwrap();
            let mut best = self.root;
            let mut best_cost = self.union(query_node, best).surface_area();
            search_queue.push_back(SSearch{
                node_handle: best,
                inherited_cost: best_cost - self.nodes.get_unchecked(best).bounds().unwrap().surface_area(),
            });

            while let Some(cur_search) = search_queue.pop_front() {
                let direct_cost = self.union(query_node, cur_search.node_handle).surface_area();
                let total_cost = direct_cost + cur_search.inherited_cost;
                if total_cost < best_cost {
                    best = cur_search.node_handle;
                    best_cost = total_cost;
                }

                if let ENode::Internal(internal) = self.nodes.get_unchecked(cur_search.node_handle) {
                    let cur_node_sa = internal.bounds.surface_area();
                    let new_inherited_cost = total_cost - cur_node_sa;
                    let children_inherited_cost = new_inherited_cost + cur_search.inherited_cost;

                    let lower_bound_cost = cur_node_sa + children_inherited_cost;

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
            node_count: 0,
            root: SPoolHandle::default(),
        }
    }

    pub fn insert(&mut self, owner: SPoolHandle, bounds: &SAABB) -> SPoolHandle {
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

        if self.node_count == 0 {
            self.root = leaf_handle;
            return leaf_handle;
        }

        // -- Step 1: find the best sibling for the new leaf
        let sibling_handle = self.find_best_sibling(leaf_handle);

        // -- Step 2: create a new parent
        let old_parent_handle = self.nodes.get_unchecked(sibling_handle).parent();

        let new_parent_handle = self.nodes.alloc().unwrap();
        {
            let new_bounds = SAABB::union(
                self.nodes.get_unchecked(sibling_handle).bounds().unwrap(),
                bounds,
            );
            let new_parent = self.nodes.get_mut_unchecked(new_parent_handle);
            *new_parent = ENode::Internal(SInternalNode{
                bounds: new_bounds,
                parent: old_parent_handle,
                child1: sibling_handle,
                child2: leaf_handle,
            });
        }

        self.nodes.get_mut_unchecked(sibling_handle).set_parent(new_parent_handle);
        self.nodes.get_mut_unchecked(leaf_handle).set_parent(new_parent_handle);

        if old_parent_handle.valid() {
            // -- sibling was not the root

            if let ENode::Internal(internal) = self.nodes.get_mut_unchecked(old_parent_handle) {
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
        let mut cur_handle = self.nodes.get_unchecked(leaf_handle).parent();
        while cur_handle.valid() {
            let (child1, child2) = {
                if let ENode::Internal(internal) = self.nodes.get_unchecked(cur_handle) {
                    (internal.child1, internal.child2)
                }
                else {
                    break_assert!(false);
                    (SPoolHandle::default(), SPoolHandle::default())
                }
            };

            let mut new_bounds = SAABB::default();
            if child1.valid() {
                new_bounds = SAABB::union(&new_bounds, &self.nodes.get_unchecked(child1).bounds().unwrap());
            }
            if child2.valid() {
                new_bounds = SAABB::union(&new_bounds, &self.nodes.get_unchecked(child2).bounds().unwrap());
            }
            self.nodes.get_mut_unchecked(cur_handle).set_bounds(&new_bounds);
            cur_handle = self.nodes.get_unchecked(cur_handle).parent();
        }

        leaf_handle
    }
}