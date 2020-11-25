// -- std includes
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use allocate::{SMemVec, TMemAllocator};

use bvh;
use entity;
use entity_model;
use render;

pub trait TDataBucketMember : std::any::Any {
}

impl TDataBucketMember for bvh::STree<entity::SEntityHandle> {}
impl TDataBucketMember for entity::SEntityBucket {}
impl TDataBucketMember for entity_model::SBucket<'static> {}
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

    pub fn and<R: TDataBucketMember>(self, data_bucket: &SDataBucket) -> Option<SMultiRef2<T, R>> {
        let second = data_bucket.get::<R>();
        if let Some(d1) = second {
            Some(SMultiRef2{
                d0: self,
                d1,
            })
        }
        else {
            None
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

    pub fn get_bvh(&self) -> Option<SDataRef<bvh::STree<entity::SEntityHandle>>> {
        self.get::<bvh::STree<entity::SEntityHandle>>()
    }

    pub fn get_entities(&self) -> Option<SDataRef<entity::SEntityBucket>> {
        self.get::<entity::SEntityBucket>()
    }

    pub fn get_renderer(&self) -> Option<SDataRef<render::SRender<'static>>> {
        self.get::<render::SRender>()
    }
}

// -- ugly helpers

#[allow(dead_code)]
pub struct SMultiRef2<T0, T1> {
    d0: SDataRef<T0>,
    d1: SDataRef<T1>,
}

#[allow(dead_code)]
pub struct SMultiRef3<T0, T1, T2> {
    d0: SDataRef<T0>,
    d1: SDataRef<T1>,
    d2: SDataRef<T2>,
}

#[allow(dead_code)]
pub struct SMultiRef4<T0, T1, T2, T3> {
    d0: SDataRef<T0>,
    d1: SDataRef<T1>,
    d2: SDataRef<T2>,
    d3: SDataRef<T3>,
}

#[allow(dead_code)]
impl<T0, T1> SMultiRef2<T0, T1> {
    pub fn and<T2: TDataBucketMember>(self, data_bucket: &SDataBucket) -> Option<SMultiRef3<T0, T1, T2>> {
        let third = data_bucket.get::<T2>();
        if let Some(d2) = third {
            Some(SMultiRef3{
                d0: self.d0,
                d1: self.d1,
                d2,
            })
        }
        else {
            None
        }
    }

    pub fn with_cc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&T0, &T1) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");

        let data0 = rc0.borrow();
        let data1 = rc1.borrow();
        function(data0.deref(), data1.deref())
    }

    pub fn with_mc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &T1) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let data1 = rc1.borrow();
        function(data0.deref_mut(), data1.deref())
    }

    pub fn with_mm<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        function(data0.deref_mut(), data1.deref_mut())
    }
}

#[allow(dead_code)]
impl<T0, T1, T2> SMultiRef3<T0, T1, T2> {
    pub fn and<T3: TDataBucketMember>(self, data_bucket: &SDataBucket) -> Option<SMultiRef4<T0, T1, T2, T3>> {
        let fourth = data_bucket.get::<T3>();
        if let Some(d3) = fourth {
            Some(SMultiRef4{
                d0: self.d0,
                d1: self.d1,
                d2: self.d2,
                d3,
            })
        }
        else {
            None
        }
    }

    pub fn with_ccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&T0, &T1, &T2) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");

        let data0 = rc0.borrow();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        function(data0.deref(), data1.deref(), data2.deref())
    }

    pub fn with_mcc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &T1, &T2) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        function(data0.deref_mut(), data1.deref(), data2.deref())
    }

    pub fn with_mmc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &T2) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let data2 = rc2.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref())
    }

    pub fn with_mmm<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut())
    }
}

#[allow(dead_code)]
impl<T0, T1, T2, T3> SMultiRef4<T0, T1, T2, T3> {
    pub fn with_cccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&T0, &T1, &T2, &T3) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.data.upgrade().expect("dropped data bucket before ref!");

        let data0 = rc0.borrow();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        function(data0.deref(), data1.deref(), data2.deref(), data3.deref())
    }

    pub fn with_mccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &T1, &T2, &T3) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        function(data0.deref_mut(), data1.deref(), data2.deref(), data3.deref())
    }

    pub fn with_mmcc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &T2, &T3) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref(), data3.deref())
    }

    pub fn with_mmmc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2, &T3) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        let data3 = rc3.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut(), data3.deref())
    }

    pub fn with_mmmm<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2, &mut T3) -> Ret
    {
        let rc0 = self.d0.data.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.data.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.data.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.data.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        let mut data3 = rc3.borrow_mut();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut(), data3.deref_mut())
    }
}
