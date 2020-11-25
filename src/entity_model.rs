use allocate::{SMemVec, TMemAllocator};
use entity::{SEntityHandle};
use bvh;
use model::SModel;

pub struct SBucket<'a> {
    pub owners: SMemVec<'a, SEntityHandle>,
    pub models: SMemVec<'a, SModel>,
    pub bvh_entries: SMemVec<'a, Option<bvh::SNodeHandle>>,
}

pub type SHandle = usize;

impl<'a> SBucket<'a> {
    pub fn new(allocator: &'a dyn TMemAllocator, max_entries: usize) -> Result<Self, &'static str> {
        Ok(Self {
            owners: SMemVec::new(allocator, max_entries, 0)?,
            models: SMemVec::new(allocator, max_entries, 0)?,
            bvh_entries: SMemVec::new(allocator, max_entries, 0)?,
        })
    }

    pub fn add_instance(&mut self, entity: SEntityHandle, model: SModel) -> Result<SHandle, &'static str> {
        self.owners.push(entity);
        self.models.push(model);
        self.bvh_entries.push(None);

        assert!(self.owners.len() == self.models.len() && self.owners.len() == self.bvh_entries.len());

        Ok(self.owners.len() - 1)
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

    pub fn get_bvh_entry(&self, handle: SHandle) -> Option<bvh::SNodeHandle> {
        self.bvh_entries[handle]
    }
}