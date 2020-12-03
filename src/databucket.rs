// -- std includes
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use allocate::{SAllocatorRef};
use collections::{SVec};

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

pub trait TDataBucketMember : std::any::Any {
}

impl TDataBucketMember for bvh::STree<entity::SEntityHandle> {}
impl TDataBucketMember for entity::SEntityBucket {}
impl TDataBucketMember for render::SRender {}
impl TDataBucketMember for animation::SAnimationLoader {}
impl TDataBucketMember for game_mode::SGameMode {}
impl TDataBucketMember for camera::SDebugFPCamera {}
impl TDataBucketMember for input::SInput {}
impl TDataBucketMember for gjk::SGJKDebug {}
impl TDataBucketMember for editmode::SEditModeInput {}

// -- "components"
impl TDataBucketMember for entity_animation::SBucket {}
impl TDataBucketMember for entity_model::SBucket {}

struct SData {
    type_id: std::any::TypeId,
    data: Rc<dyn std::any::Any>,
}

pub struct SDataRefBuilder<'bucket, T> {
    bucket: &'bucket SDataBucket,
    result: SDataRef<T>,
}

pub struct SDataRef<T> {
    data: Weak<RefCell<T>>,
}

pub struct SDataBucket {
    entries: SVec<SData>,
}

impl SData {
    fn new<T: TDataBucketMember>(d: T) -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            data: Rc::new(RefCell::new(d)),
        }
    }

    fn is<T: TDataBucketMember>(&self) -> bool {
        self.type_id == std::any::TypeId::of::<T>()
    }

    fn get_weak<T: TDataBucketMember>(&self) -> Weak<RefCell<T>> {
        let typed = self.data.clone()
            .downcast::<RefCell<T>>()
            .expect("shouldn't call this without checking type");

        Rc::downgrade(&typed)
    }
}

impl SDataBucket {
    pub fn new(max_entries: usize, allocator: &SAllocatorRef) -> Self {
        Self {
            entries: SVec::new(allocator, max_entries, 0).unwrap(),
        }
    }

    pub fn add<T: TDataBucketMember>(&mut self, member: T) {
        self.entries.push(SData::new(member));
    }

    fn get_entry<T: TDataBucketMember>(&self) -> Option<&SData> {
        for entry in self.entries.as_slice() {
            if entry.is::<T>() {
                return Some(entry);
            }
        }

        None
    }

    pub fn get<T: TDataBucketMember>(&self) -> SDataRefBuilder<T> {
        let entry = self.get_entry::<T>().expect("invalid entry");
        SDataRefBuilder::new(self, entry)
    }

    pub fn get_entities(&self) -> SDataRefBuilder<entity::SEntityBucket> {
        self.get::<entity::SEntityBucket>()
    }

    pub fn get_renderer(&self) -> SDataRefBuilder<render::SRender> {
        self.get::<render::SRender>()
    }
}

impl<'bucket, T: TDataBucketMember> SDataRefBuilder<'bucket, T> {
    fn new(bucket: &'bucket SDataBucket, data: &SData) -> Self {
        Self{
            bucket,
            result: SDataRef{
                data: data.get_weak(),
            },
        }
    }

    pub fn and<T1: TDataBucketMember>(self) -> SMultiRefBuilder2<'bucket, T, T1> {
        let d1 = self.bucket.get_entry::<T1>().expect("invalid entry");
        SMultiRefBuilder2::new_from_1(self, d1)
    }

    pub fn build(self) -> SDataRef<T> {
        self.result
    }
}

impl<'bucket, T0> Deref for SDataRefBuilder<'bucket, T0> {
    type Target = SDataRef<T0>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

impl<T: TDataBucketMember> SDataRef<T> {
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

// -- ugly helpers

#[allow(dead_code)]
pub struct SMultiRefBuilder2<'bucket, T0, T1> {
    bucket: &'bucket SDataBucket,
    result: SMultiRef2<T0, T1>,
}

#[allow(dead_code)]
pub struct SMultiRef2<T0, T1> {
    d0: Weak<RefCell<T0>>,
    d1: Weak<RefCell<T1>>,
}

#[allow(dead_code)]
pub struct SMultiRefBuilder3<'bucket, T0, T1, T2> {
    bucket: &'bucket SDataBucket,
    result: SMultiRef3<T0, T1, T2>,
}

#[allow(dead_code)]
pub struct SMultiRef3<T0, T1, T2> {
    d0: Weak<RefCell<T0>>,
    d1: Weak<RefCell<T1>>,
    d2: Weak<RefCell<T2>>,
}

#[allow(dead_code)]
pub struct SMultiRefBuilder4<'bucket, T0, T1, T2, T3> {
    bucket: &'bucket SDataBucket,
    result: SMultiRef4<T0, T1, T2, T3>,
}

#[allow(dead_code)]
pub struct SMultiRef4<T0, T1, T2, T3> {
    d0: Weak<RefCell<T0>>,
    d1: Weak<RefCell<T1>>,
    d2: Weak<RefCell<T2>>,
    d3: Weak<RefCell<T3>>,
}

#[allow(dead_code)]
pub struct SMultiRefBuilder5<'bucket, T0, T1, T2, T3, T4> {
    bucket: &'bucket SDataBucket,
    result: SMultiRef5<T0, T1, T2, T3, T4>,
}

#[allow(dead_code)]
pub struct SMultiRef5<T0, T1, T2, T3, T4> {
    d0: Weak<RefCell<T0>>,
    d1: Weak<RefCell<T1>>,
    d2: Weak<RefCell<T2>>,
    d3: Weak<RefCell<T3>>,
    d4: Weak<RefCell<T4>>,
}

#[allow(dead_code)]
impl<'bucket, T0, T1: TDataBucketMember> SMultiRefBuilder2<'bucket, T0, T1> {
    fn new_from_1(prev: SDataRefBuilder<'bucket, T0>, last: &SData) -> Self {
        Self{
            bucket: prev.bucket,
            result: SMultiRef2::<T0, T1> {
                d0: prev.result.data,
                d1: last.get_weak(),
            },
        }
    }

    pub fn and<T2: TDataBucketMember>(self) -> SMultiRefBuilder3<'bucket, T0, T1, T2> {
        let last = self.bucket.get_entry::<T2>().expect("invalid entry");
        SMultiRefBuilder3::new_from_2(self, last)
    }

    pub fn build(self) -> SMultiRef2<T0, T1> {
        self.result
    }
}

impl<'bucket, T0, T1> Deref for SMultiRefBuilder2<'bucket, T0, T1> {
    type Target = SMultiRef2<T0, T1>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[allow(dead_code)]
impl<T0, T1: TDataBucketMember> SMultiRef2<T0, T1> {
    pub fn with_cc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&T0, &T1) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");

        let data0 = rc0.borrow();
        let data1 = rc1.borrow();
        function(data0.deref(), data1.deref())
    }

    pub fn with_mc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &T1) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let data1 = rc1.borrow();
        function(data0.deref_mut(), data1.deref())
    }

    pub fn with_mm<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        function(data0.deref_mut(), data1.deref_mut())
    }
}

impl<'bucket, T0, T1, T2: TDataBucketMember> SMultiRefBuilder3<'bucket, T0, T1, T2> {
    fn new_from_2(prev: SMultiRefBuilder2<'bucket, T0, T1>, last: &SData) -> Self {
        Self{
            bucket: prev.bucket,
            result: SMultiRef3::<T0, T1, T2> {
                d0: prev.result.d0,
                d1: prev.result.d1,
                d2: last.get_weak(),
            },
        }
    }

    pub fn and<T3: TDataBucketMember>(self) -> SMultiRefBuilder4<'bucket, T0, T1, T2, T3> {
        let last = self.bucket.get_entry::<T3>().expect("invalid entry");
        SMultiRefBuilder4::new_from_3(self, last)
    }

    pub fn build(self) -> SMultiRef3<T0, T1, T2> {
        self.result
    }
}

impl<'bucket, T0, T1, T2> Deref for SMultiRefBuilder3<'bucket, T0, T1, T2> {
    type Target = SMultiRef3<T0, T1, T2>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[allow(dead_code)]
impl<T0, T1, T2: TDataBucketMember> SMultiRef3<T0, T1, T2> {
    pub fn with_ccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&T0, &T1, &T2) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");

        let data0 = rc0.borrow();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        function(data0.deref(), data1.deref(), data2.deref())
    }

    pub fn with_mcc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &T1, &T2) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        function(data0.deref_mut(), data1.deref(), data2.deref())
    }

    pub fn with_mmc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &T2) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let data2 = rc2.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref())
    }

    pub fn with_mmm<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut())
    }
}

impl<'bucket, T0, T1, T2, T3: TDataBucketMember> SMultiRefBuilder4<'bucket, T0, T1, T2, T3> {
    fn new_from_3(prev: SMultiRefBuilder3<'bucket, T0, T1, T2>, last: &SData) -> Self {
        Self{
            bucket: prev.bucket,
            result: SMultiRef4::<T0, T1, T2, T3> {
                d0: prev.result.d0,
                d1: prev.result.d1,
                d2: prev.result.d2,
                d3: last.get_weak(),
            },
        }
    }

    pub fn and<T4: TDataBucketMember>(self) -> SMultiRefBuilder5<'bucket, T0, T1, T2, T3, T4> {
        let last = self.bucket.get_entry::<T4>().expect("invalid entry");
        SMultiRefBuilder5::new_from_4(self, last)
    }

    pub fn build(self) -> SMultiRef4<T0, T1, T2, T3> {
        self.result
    }
}

impl<'bucket, T0, T1, T2, T3> Deref for SMultiRefBuilder4<'bucket, T0, T1, T2, T3> {
    type Target = SMultiRef4<T0, T1, T2, T3>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[allow(dead_code)]
impl<T0, T1, T2, T3: TDataBucketMember> SMultiRef4<T0, T1, T2, T3> {
    pub fn with_cccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&T0, &T1, &T2, &T3) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");

        let data0 = rc0.borrow();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        function(data0.deref(), data1.deref(), data2.deref(), data3.deref())
    }

    pub fn with_mccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &T1, &T2, &T3) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let data1 = rc1.borrow();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        function(data0.deref_mut(), data1.deref(), data2.deref(), data3.deref())
    }

    pub fn with_mmcc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &T2, &T3) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref(), data3.deref())
    }

    pub fn with_mmmc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2, &T3) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        let data3 = rc3.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut(), data3.deref())
    }

    pub fn with_mmmm<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2, &mut T3) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        let mut data3 = rc3.borrow_mut();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut(), data3.deref_mut())
    }
}

impl<'bucket, T0, T1, T2, T3, T4: TDataBucketMember> SMultiRefBuilder5<'bucket, T0, T1, T2, T3, T4> {
    fn new_from_4(prev: SMultiRefBuilder4<'bucket, T0, T1, T2, T3>, last: &SData) -> Self {
        Self{
            bucket: prev.bucket,
            result: SMultiRef5::<T0, T1, T2, T3, T4> {
                d0: prev.result.d0,
                d1: prev.result.d1,
                d2: prev.result.d2,
                d3: prev.result.d3,
                d4: last.get_weak(),
            },
        }
    }

    pub fn build(self) -> SMultiRef5<T0, T1, T2, T3, T4> {
        self.result
    }
}

impl<'bucket, T0, T1, T2, T3, T4> Deref for SMultiRefBuilder5<'bucket, T0, T1, T2, T3, T4> {
    type Target = SMultiRef5<T0, T1, T2, T3, T4>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[allow(dead_code)]
impl<T0, T1, T2, T3, T4: TDataBucketMember> SMultiRef5<T0, T1, T2, T3, T4> {
    pub fn with_mmccc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &T2, &T3, &T4) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");
        let rc4 = self.d4.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let data2 = rc2.borrow();
        let data3 = rc3.borrow();
        let data4 = rc4.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref(), data3.deref(), data4.deref())
    }

    pub fn with_mmmcc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2, &T3, &T4) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");
        let rc4 = self.d4.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        let data3 = rc3.borrow();
        let data4 = rc4.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut(), data3.deref(), data4.deref())
    }

    pub fn with_mmmmc<Fun, Ret>(&self, mut function: Fun) -> Ret where
    Fun: FnMut(&mut T0, &mut T1, &mut T2, &mut T3, &T4) -> Ret
    {
        let rc0 = self.d0.upgrade().expect("dropped data bucket before ref!");
        let rc1 = self.d1.upgrade().expect("dropped data bucket before ref!");
        let rc2 = self.d2.upgrade().expect("dropped data bucket before ref!");
        let rc3 = self.d3.upgrade().expect("dropped data bucket before ref!");
        let rc4 = self.d4.upgrade().expect("dropped data bucket before ref!");

        let mut data0 = rc0.borrow_mut();
        let mut data1 = rc1.borrow_mut();
        let mut data2 = rc2.borrow_mut();
        let mut data3 = rc3.borrow_mut();
        let data4 = rc4.borrow();
        function(data0.deref_mut(), data1.deref_mut(), data2.deref_mut(), data3.deref_mut(), data4.deref())
    }
}
