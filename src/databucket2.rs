// -- std includes
use std::cell::{RefCell};
//use std::ops::{Deref, DerefMut};

use allocate::{SMemVec, TMemAllocator, SMem, SMemT};

use animation;
///use camera;
use bvh;
//use editmode;
use entity;
///use entity_animation;
///use entity_model;
///use gjk;
///use input;
///use game_mode;
///use render;

#[derive(std::cmp::PartialEq)]
pub enum ETypeID {
    BVH,
    AnimationLoader,
}

pub trait TDataBucketMember<'bucket> {
    const TYPE_ID: ETypeID;
}

impl<'bucket> TDataBucketMember<'bucket> for bvh::STree<entity::SEntityHandle> {
    const TYPE_ID : ETypeID = ETypeID::BVH;
}
impl<'bucket, 'a> TDataBucketMember<'bucket> for animation::SAnimationLoader<'a>
    where 'a: 'bucket
{
    const TYPE_ID : ETypeID = ETypeID::AnimationLoader;
}

struct SEntry<'alloc> {
    type_id: ETypeID,
    data: RefCell<SMem<'alloc>>,
}

pub struct SDataBucket<'alloc> {
    allocator: &'alloc dyn TMemAllocator,
    entries: SMemVec<'alloc, SEntry<'alloc>>,
}

pub struct SDataRef<'bucket, 'alloc, T> {
    bucket: &'bucket SDataBucket<'alloc>,
    data: &'bucket RefCell<SMem<'alloc>>,
    phantom: std::marker::PhantomData<T>,
}

impl<'alloc> Drop for SEntry<'alloc> {
    fn drop(&mut self) {
        match self.type_id {
            ETypeID::BVH => {

            },
            ETypeID::AnimationLoader => {

            },
        }
    }
}

impl<'alloc> SDataBucket<'alloc> {
    pub fn new(max_entries: usize, allocator: &'alloc dyn TMemAllocator) -> Self {
        Self {
            allocator,
            entries: SMemVec::new(allocator, max_entries, 0).unwrap(),
        }
    }

    pub fn add<'bucket, T: TDataBucketMember<'bucket>>(&'bucket mut self, member: T) {
        let t_mem = SMemT::new(self.allocator, member).unwrap();
        let entry = unsafe { SEntry{
            type_id: T::TYPE_ID,
            data: RefCell::new(t_mem.into_raw()),
        }};
        self.entries.push(entry);
    }

    fn get_entry_data<'bucket, T: TDataBucketMember<'bucket>>(&'bucket self) -> Option<&'bucket RefCell<SMem<'alloc>>> {
        for entry in self.entries.as_slice() {
            if entry.type_id == T::TYPE_ID {
                return Some(&entry.data);
            }
        }

        None
    }

    pub fn get<'bucket, T: TDataBucketMember<'bucket>>(&'bucket self) -> SDataRef<'bucket, 'alloc, T> {
        let entry = self.get_entry_data::<T>().expect("invalid entry");
        SDataRef {
            bucket: self,
            data: &entry,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'bucket, 'alloc, T: TDataBucketMember<'bucket>> SDataRef<'bucket, 'alloc, T> {
    pub fn with<F, R>(&self, mut function: F) -> R where
    F: FnMut(&T) -> R
    {
        let mem = self.data.borrow();
        let casted = unsafe { mem.as_ref_typed::<T>() };
        function(casted)
    }
}
