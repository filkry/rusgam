// -- std includes
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use allocate::{SMemVec, TMemAllocator, SMem, SMemT};

use animation;
use camera;
use bvh;
use editmode;
use entity;
use entity_animation;
use entity_model;
use gjk;
use input;
use game_mode;
use render;

#[derive(std::cmp::PartialEq)]
pub enum ETypeID {
    BVH,
}

pub trait TDataBucketMember {
    const TYPE_ID: ETypeID;
}

impl TDataBucketMember for bvh::STree<entity::SEntityHandle> {
    const TYPE_ID : ETypeID = ETypeID::BVH;
}

struct SEntry<'alloc> {
    type_id: ETypeID,
    data: Rc<SMem<'alloc>>,
}

pub struct SDataBucket<'alloc> {
    allocator: &'alloc dyn TMemAllocator,
    entries: SMemVec<'alloc, SEntry<'alloc>>,
}

impl<'alloc> SEntry<'alloc> {
    pub fn new<T: TDataBucketMember>(member: SMemT<'alloc, T>) -> Self {
        let raw = unsafe { member.into_raw() };

        Self {
            type_id: T::TYPE_ID,
            data: Rc::new(raw),
        }
    }
}

impl<'alloc> SDataBucket<'alloc> {
    pub fn add<T: TDataBucketMember>(&mut self, member: T) {
        let t_mem = SMemT::new(self.allocator, member).unwrap();
        self.entries.push(SEntry::new(t_mem));
    }

    fn get_entry<T: TDataBucketMember>(&self) -> Option<&SEntry> {
        for entry in self.entries.as_slice() {
            if entry.type_id == T::TYPE_ID {
                return Some(entry);
            }
        }

        None
    }

    pub fn get<T: TDataBucketMember>(&self) -> SDataRef<T> {
        let entry = self.get_entry::<T>().expect("invalid entry");
        SDataRef::new(self, entry)
    }
}