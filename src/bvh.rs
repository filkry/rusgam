
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

struct Tree {
    nodes: SPool<ENode>,
    node_count: u32,
    root: SPoolHandle,
}

impl Tree {
    pub fn insert(&mut self, owner: SPoolHandle, bounds: &SAABB) -> SPoolHandle {
        let node_handle = self.nodes.alloc().unwrap();

        // -- initialize node
        {
            let node = self.nodes.get_mut(node_handle).unwrap();
            node = ENode::Leaf(SLeafNode{
                bounds: bounds.clone(),
                parent: Default::default(),
                owner: owner,
            });
        }

        if self.node_count == 0 {
            self.root = node_handle;
            return;
        }

        // -- Step 1: find the best sibling for the new leaf
        let sibling_handle = {

        };

        // -- Step 2: create a new parent
        let old_parent_handle = self.nodes.get_unchecked(sibling_handle).parent;

        let new_parent_handle = self.nodes.alloc().unwrap();
        {
            let new_parent = nodes.get_mut_unchecked(new_parent_handle);
            new_parent = ENode::Internal(SInternalNode{
                bounds: SAABB::union(sibling_bounds, bounds),
                parent: old_parent_handle,
                child1: sibling_handle,
                child2: node_handle,
            });
        }

        self.nodes.get_mut_unchecked(sibling_handle).parent = new_parent_handle;
        self.nodes.get_mut_unchecked(node_handle).parent = new_parent_handle;

        if old_parent_handle.valid() {
            // -- sibling was not the root

            // -- update old parent's child to the new parent
            if nodes.get_unchecked(old_parent_handle).child1 == sibling_handle {
                self.nodes.get_mut_unchecked(old_parent_handle).child1 = new_parent;
            }
            else {
                self.nodes.get_mut_unchecked(old_parent_handle).child2 = new_parent;
            }
        }
        else {
            // -- sibling was the root, new parent is now root
            self.root = new_parent_handle;
        }

        // -- Step 3: walk up, refitting AABBs

    }
}