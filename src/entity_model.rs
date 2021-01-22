use crate::allocate::{SAllocatorRef};
use crate::collections::{SVec};
use crate::entity::{SEntityHandle};
use crate::bvh;
use crate::model::SModel;

pub struct SBucket {
    pub owners: SVec<SEntityHandle>,
    pub models: SVec<SModel>,
    pub bvh_entries: SVec<Option<bvh::SNodeHandle>>,
}

pub type SHandle = usize;

impl SBucket {
    pub fn new(allocator: &SAllocatorRef, max_entries: usize) -> Result<Self, &'static str> {
        Ok(Self {
            owners: SVec::new(allocator, max_entries, 0)?,
            models: SVec::new(allocator, max_entries, 0)?,
            bvh_entries: SVec::new(allocator, max_entries, 0)?,
        })
    }

    pub fn add_instance(&mut self, entity: SEntityHandle, model: SModel) -> Result<SHandle, &'static str> {
        self.owners.push(entity);
        self.models.push(model);
        self.bvh_entries.push(None);

        assert!(self.owners.len() == self.models.len() && self.owners.len() == self.bvh_entries.len());

        Ok(self.owners.len() - 1)
    }

    #[allow(unused_variables)]
    pub fn purge_entities(&mut self, entities: &[SEntityHandle]) {
        let mut i = 0;
        while i < self.owners.len() {
            let mut purge = false;
            for entity in entities {
                if *entity == self.owners[i] {
                    purge = true;
                    break;
                }
            }

            if purge {
                self.owners.swap_remove(i);
                self.models.swap_remove(i);
                self.bvh_entries.swap_remove(i);
            }
            else {
                i = i + 1;
            }
        }
    }

    pub fn handle_for_entity(&self, entity: SEntityHandle) -> Option<SHandle> {
        for i in 0..self.owners.len() {
            if self.owners[i] == entity {
                return Some(i);
            }
        }

        None
    }

    pub fn get_entity(&self, handle: SHandle) -> SEntityHandle {
        self.owners[handle]
    }

    pub fn get_model(&self, handle: SHandle) -> &SModel {
        &self.models[handle]
    }

    pub fn set_bvh_entry(&mut self, handle: SHandle, entry: bvh::SNodeHandle) {
        self.bvh_entries[handle] = Some(entry);
    }

    pub fn get_bvh_entry(&self, handle: SHandle) -> Option<bvh::SNodeHandle> {
        self.bvh_entries[handle]
    }
}