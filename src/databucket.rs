// -- std includes
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use allocate::{SMemVec, TMemAllocator};
use entity;
use bvh;

struct SData<T> {
    data: Rc<RefCell<T>>, // $$$FRK(TODO): write Rc+Weak that can go in my own allocators
}

pub struct SDataRef<T> {
    data: Weak<RefCell<T>>,
}

enum EDataEntry {
    BVH(SData<bvh::STree>),
    Entities(SData<entity::SEntityBucket>),
}

pub struct SDataBucket<'a> {
    entries: SMemVec<'a, EDataEntry>,
}

pub struct SDataBucketOwner<'a> {
    bucket: Rc<RefCell<SDataBucket<'a>>>,
}

pub struct SDataBucketRef<'a> {
    bucket: Weak<RefCell<SDataBucket<'a>>>,
}

impl<T> SData<T> {
    pub fn new(d: T) -> Self {
        Self {
            data: Rc::new(RefCell::new(d)),
        }
    }

    pub fn make_ref(&self) -> SDataRef<T> {
        SDataRef{
            data: Rc::downgrade(&self.data),
        }
    }
}

impl<T> SDataRef<T> {
    pub fn with<F>(&self, mut function: F) where
    F: FnMut(&T)
    {
        let rc = self.data.upgrade().expect("dropped data bucket before ref!");
        let data = rc.borrow();
        function(data.deref());
    }

    pub fn with_mut<F>(&self, mut function: F) where
    F: FnMut(&mut T)
    {
        let rc = self.data.upgrade().expect("dropped data bucket before ref!");
        let mut data = rc.borrow_mut();
        function(data.deref_mut());
    }
}

impl<'a> SDataBucket<'a> {
    pub fn new(max_entries: usize, allocator: &'a dyn TMemAllocator) -> Self {
        Self {
            entries: SMemVec::new(allocator, max_entries, 0).unwrap(),
        }
    }

    pub fn add_bvh(&mut self, bvh: bvh::STree) {
        self.entries.push(
            EDataEntry::BVH(SData::<bvh::STree>::new(bvh))
        );
    }

    pub fn add_entities(&mut self, entities: entity::SEntityBucket) {
        self.entries.push(
            EDataEntry::Entities(SData::<entity::SEntityBucket>::new(entities))
        );
    }

    pub fn get_bvh(&self) -> Option<SDataRef<bvh::STree>> {
        for entry in self.entries.as_slice() {
            match entry {
                EDataEntry::BVH(data) => {
                    return Some(data.make_ref())
                },
                _ => {}
            }
        }

        None
    }

    pub fn get_entities(&self) -> Option<SDataRef<entity::SEntityBucket>> {
        for entry in self.entries.as_slice() {
            match entry {
                EDataEntry::Entities(data) => {
                    return Some(data.make_ref())
                },
                _ => {}
            }
        }

        None
    }
}

impl<'a> SDataBucketOwner<'a> {
    pub fn new(max_entries: usize, allocator: &'a dyn TMemAllocator) -> Self {
        Self {
            bucket: Rc::new(RefCell::new(
                SDataBucket::new(max_entries, allocator)
            ))
        }
    }
}

impl<'a> SDataBucketRef<'a> {
    pub fn with<F>(&self, mut function: F) where
    F: FnMut(&SDataBucket<'a>)
    {
        let rc = self.bucket.upgrade().expect("dropped data bucket before ref!");
        let data = rc.borrow();
        function(data.deref());
    }

    pub fn with_mut<F>(&self, mut function: F) where
    F: FnMut(&SDataBucket<'a>)
    {
        let rc = self.bucket.upgrade().expect("dropped data bucket before ref!");
        let mut data = rc.borrow_mut();
        function(data.deref_mut());
    }
}