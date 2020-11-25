// -- std includes
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use allocate::{SMemVec, TMemAllocator};

use bvh;
use entity;
use render;

pub trait TDataBucketMember : std::any::Any {
}

impl TDataBucketMember for bvh::STree {}
impl TDataBucketMember for entity::SEntityBucket {}
impl TDataBucketMember for render::SRender<'static> {}

// -- $$$FRK(TODO): originally I thought I might want to keep SDataRefs around, but maybe not?
// -- If I don't, then this can become Box<RefCell> and the usage syntax can become cleaner (no
// -- SDataRef::with, ::with_mut lambda shenanigans)
struct SData {
    type_id: std::any::TypeId,
    data: Rc<dyn std::any::Any>, // $$$FRK(TODO): write Rc+Weak that can go in my own allocators
}

pub struct SDataRef<T> {
    data: Weak<RefCell<T>>,
}

pub struct SDataBucket<'a> {
    entries: SMemVec<'a, SData>,
}

impl SData {
    pub fn new<T: TDataBucketMember>(d: T) -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            data: Rc::new(RefCell::new(d)),
        }
    }

    pub fn is<T: TDataBucketMember>(&self) -> bool {
        self.type_id == std::any::TypeId::of::<T>()
    }
}

impl<T: 'static> SDataRef<T> {
    fn new(data: &SData) -> Self {
        let typed = data.data.clone().downcast::<RefCell<T>>().expect("shouldn't call this without checking type");
        Self{
            data: Rc::downgrade(&typed),
        }
    }

    pub fn with<F, R>(&self, mut function: F) -> R where
    F: FnMut(&T) -> R
    {
        let rc = self.data.upgrade().expect("dropped data bucket before ref!");
        let data = rc.borrow();
        function(data.deref())
    }

    pub fn with_mut<F, R>(&self, mut function: F) -> R where
    F: FnMut(&mut T) -> R
    {
        let rc = self.data.upgrade().expect("dropped data bucket before ref!");
        let mut data = rc.borrow_mut();
        function(data.deref_mut())
    }
}

impl<'a> SDataBucket<'a> {
    pub fn new(max_entries: usize, allocator: &'a dyn TMemAllocator) -> Self {
        Self {
            entries: SMemVec::new(allocator, max_entries, 0).unwrap(),
        }
    }

    pub fn add<T: TDataBucketMember>(&mut self, member: T) {
        self.entries.push(SData::new(member));
    }

    pub fn get<T: TDataBucketMember>(&self) -> Option<SDataRef<T>> {
        for entry in self.entries.as_slice() {
            if entry.is::<T>() {
                return Some(SDataRef::<T>::new(entry));
            }
        }

        None
    }

    pub fn get_bvh(&self) -> Option<SDataRef<bvh::STree>> {
        self.get::<bvh::STree>()
    }

    pub fn get_entities(&self) -> Option<SDataRef<entity::SEntityBucket>> {
        self.get::<entity::SEntityBucket>()
    }

    pub fn get_renderer(&self) -> Option<SDataRef<render::SRender<'static>>> {
        self.get::<render::SRender>()
    }
}