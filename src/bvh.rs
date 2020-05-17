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
    Free(SAABB),
    Leaf(SLeafNode),
    Internal(SInternalNode),
}

struct Tree {
    nodes: SPool<ENode>,
    node_count: u32,
    root: SPoolHandle,
}

impl ENode {
    pub fn parent(&self) -> SPoolHandle {
        match self {
            Self::Free(_) => {
                break_assert!(false);
                SPoolHandle::default()
            },
            Self::Leaf(leaf) => leaf.parent,
            Self::Internal(internal) => internal.parent,
        }
    }

    pub fn bounds(&self) -> Option<&SAABB> {
        match self {
            Self::Free(dummy_bounds) => {
                break_assert!(false);
                None
            },
            Self::Leaf(leaf) => Some(&leaf.bounds),
            Self::Internal(internal) => Some(&internal.bounds),
        }
    }

    pub fn set_parent(&mut self, new_parent: SPoolHandle) {
        match self {
            Self::Free(_) => {
                break_assert!(false);
            },
            Self::Leaf(leaf) => { leaf.parent = new_parent },
            Self::Internal(internal) => { internal.parent = new_parent },
        }
    }

    pub fn set_bounds(&mut self, new_bounds: &SAABB) {
        match self {
            Self::Free(_) => {
                break_assert!(false);
            },
            Self::Leaf(leaf) => { leaf.bounds = new_bounds.clone() },
            Self::Internal(internal) => { internal.bounds = new_bounds.clone() },
        }
    }
}

impl Tree {
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
        let sibling_handle = {
            break_assert!(false);
            SPoolHandle::default()
        };

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