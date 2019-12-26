#![allow(dead_code)]

//use std::iter::IntoIterator;
use std::cell::RefCell;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use utils::align_up;

pub static SYSTEM_ALLOCATOR: SSystemAllocator = SSystemAllocator {};

thread_local! {
    pub static TEMP_ALLOCATOR : RefCell<SGenAllocator<SLinearAllocator<'static>>> =
        RefCell::new(
            SGenAllocator::new(
                SLinearAllocator::new(&SYSTEM_ALLOCATOR, 4 * 1024 * 1024, 8).unwrap()));

    pub static STACK_ALLOCATOR : SStackAllocator<'static> =
        SStackAllocator::new(&SYSTEM_ALLOCATOR, 4 * 1024 * 1024, 8).unwrap();
}

pub trait TMemAllocator {
    // -- things implementing TMemAllocator should rely on internal mutability, since their
    // -- allocations will have a reference to them
    fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str>;
    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<SMem, &'static str>;

    // -- unsafe because it doesn't consume the SMem
    fn free(&self, existing_allocation: SMem) -> Result<(), &'static str>;
    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str>;

    unsafe fn reset(&self);
}

pub struct SSystemAllocator {}

impl TMemAllocator for SSystemAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str> {
        let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
        let data = unsafe { std::alloc::alloc(layout) as *mut u8 };

        if data == std::ptr::null_mut() {
            return Err("failed to allocate");
        }

        Ok(SMem {
            data: data,
            size: size,
            alignment: align,
            allocator: self,
        })
    }

    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<SMem, &'static str> {
        let layout = std::alloc::Layout::from_size_align(
            existing_allocation.size,
            existing_allocation.alignment,
        )
        .unwrap();
        let data =
            unsafe { std::alloc::realloc(existing_allocation.data, layout, new_size) as *mut u8 };

        if data == std::ptr::null_mut() {
            // -- failed to re-alloc, free memory and run
            self.free(existing_allocation)?;
            return Err("failed to re-alloc");
        }

        Ok(SMem {
            data: data,
            size: new_size,
            alignment: existing_allocation.alignment,
            allocator: self,
        })
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let layout = std::alloc::Layout::from_size_align(
            existing_allocation.size,
            existing_allocation.alignment,
        )
        .unwrap();

        println!("maybe?");

        std::alloc::dealloc(existing_allocation.data, layout);

        existing_allocation.data = std::ptr::null_mut();
        existing_allocation.size = 0;

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    unsafe fn reset(&self) {}
}

struct SLinearAllocatorData<'a> {
    raw: SMem<'a>,
    cur_offset: usize,
    allow_realloc: bool,
}

pub struct SLinearAllocator<'a> {
    data: RefCell<SLinearAllocatorData<'a>>,
}

impl<'a> SLinearAllocator<'a> {
    pub fn new(
        parent: &'a dyn TMemAllocator,
        size: usize,
        align: usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            data: RefCell::new(SLinearAllocatorData {
                raw: parent.alloc(size, align)?,
                cur_offset: 0,
                allow_realloc: false,
            }),
        })
    }
}

impl<'a> TMemAllocator for SLinearAllocator<'a> {
    fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str> {
        let mut data = self.data.borrow_mut();

        if (data.raw.data as usize) % align != 0 {
            panic!("Currently don't support different alignments.");
        }

        let aligned_offset = align_up(data.cur_offset, align);
        let aligned_size = align_up(size, align);

        if (aligned_offset + aligned_size) > data.raw.size {
            return Err("Out of memory");
        }

        let result = SMem {
            data: unsafe { data.raw.data.add(aligned_offset) },
            size: aligned_size,
            alignment: align,
            allocator: self,
        };

        data.cur_offset = aligned_offset + aligned_size;

        Ok(result)
    }

    fn realloc(&self, _existing_allocation: SMem, _new_size: usize) -> Result<SMem, &'static str> {
        let data = self.data.borrow_mut();

        if data.allow_realloc {
            panic!("Not implemented.")
        }

        Err("Does not allow realloc.")
    }

    unsafe fn free_unsafe(&self, _existing_allocation: &mut SMem) -> Result<(), &'static str> {
        Ok(())
    }

    fn free(&self, mut _existing_allocation: SMem) -> Result<(), &'static str> {
        Ok(())
    }

    unsafe fn reset(&self) {
        let mut data = self.data.borrow_mut();

        data.cur_offset = 0;
    }
}

struct SStackAllocatorData<'a> {
    raw: SMem<'a>,
    top_offset: usize,
}

pub struct SStackAllocator<'a> {
    data: RefCell<SStackAllocatorData<'a>>,
}

impl<'a> SStackAllocator<'a> {
    pub fn new(
        parent: &'a dyn TMemAllocator,
        size: usize,
        align: usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            data: RefCell::new(SStackAllocatorData {
                raw: parent.alloc(size, align)?,
                top_offset: 0,
            }),
        })
    }
}

// -- $$$FRK(TODO): allocators should check if they own the mem when they free!
impl<'a> TMemAllocator for SStackAllocator<'a> {
    fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str> {
        let mut data = self.data.borrow_mut();

        if (data.raw.data as usize) % align != 0 {
            panic!("Currently don't support different alignments.");
        }

        let aligned_offset = align_up(data.top_offset, align);
        let aligned_size = align_up(size, align);

        if (aligned_offset + aligned_size) > data.raw.size {
            return Err("Out of memory");
        }

        let result = SMem {
            data: unsafe { data.raw.data.add(aligned_offset) },
            size: aligned_size,
            alignment: align,
            allocator: self,
        };

        data.top_offset = aligned_offset + aligned_size;

        Ok(result)
    }

    fn realloc(&self, _existing_allocation: SMem, _new_size: usize) -> Result<SMem, &'static str> {
        panic!("Cannot re-alloc in stack allocator.")
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let mut data = self.data.borrow_mut();

        let ea_top = existing_allocation.data.add(existing_allocation.size);
        let self_top = data.raw.data.add(data.top_offset);
        if ea_top != self_top {
            panic!("Trying to free from the stack array, but not the top.");
        }

        data.top_offset = (existing_allocation.data as usize) - (data.raw.data as usize);
        existing_allocation.invalidate();

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    unsafe fn reset(&self) {
        panic!("Stack allocator does not handle reset!");
    }
}

pub struct SMem<'a> {
    data: *mut u8,
    size: usize,
    alignment: usize,
    allocator: &'a dyn TMemAllocator,
}

impl<'a> SMem<'a> {
    fn invalidate(&mut self) {
        self.data = std::ptr::null_mut();
        self.size = 0;
        self.alignment = 0;
    }
}

impl<'a> Drop for SMem<'a> {
    fn drop(&mut self) {
        let allocator = self.allocator;
        unsafe { allocator.free_unsafe(self).unwrap() };
    }
}

pub struct SMemVec<'a, T> {
    mem: SMem<'a>,
    len: usize,
    capacity: usize,
    grow_capacity: usize,

    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> SMemVec<'a, T> {
    pub fn new(
        allocator: &'a dyn TMemAllocator,
        initial_capacity: usize,
        grow_capacity: usize,
    ) -> Result<Self, &'static str> {
        let num_bytes = initial_capacity * size_of::<T>();

        Ok(Self {
            mem: allocator.alloc(num_bytes, 8)?,
            len: 0,
            capacity: initial_capacity,
            grow_capacity: grow_capacity,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn new_genned<A: TMemAllocator>(
        allocator: &'a SGenAllocator<A>,
        initial_capacity: usize,
        grow_capacity: usize,
    ) -> Result<SGenAllocation<'a, Self, A>, &'static str> {
        Ok(SGenAllocation {
            raw: Self::new(&allocator.raw, initial_capacity, grow_capacity)?,
            generation: allocator.generation(),
            temp_allocator: allocator,
        })
    }

    fn data(&self) -> *mut T {
        self.mem.data as *mut T
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            if self.grow_capacity == 0 {
                assert!(false, "Out of space, not pushing.");
                return;
            } else {
                panic!("Grow not implemented!")
            }
        }

        self.len += 1;
        let idx = self.len - 1;
        self[idx] = value;
    }
}

impl<'a, T> Deref for SMemVec<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data(), self.len) }
    }
}

impl<'a, T> DerefMut for SMemVec<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data(), self.len) }
    }
}

impl<'a, T, I: std::slice::SliceIndex<[T]>> Index<I> for SMemVec<'a, T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        //Index::index(&**self, index)
        Index::index(self.deref(), index) // relying on Index implementation of &[T]
    }
}

impl<'a, T, I: std::slice::SliceIndex<[T]>> IndexMut<I> for SMemVec<'a, T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        //IndexMut::index_mut(&mut **self, index)
        IndexMut::index_mut(self.deref_mut(), index)
    }
}

// -- this generation'd allocation wrapping is still unsafe because the generation could
// -- change while we're looking at the data - we don't guard every touch of the memory
pub struct SGenAllocator<A: TMemAllocator> {
    raw: A,
    generation: RefCell<u32>,
}

impl<A: TMemAllocator> SGenAllocator<A> {
    pub fn new(raw: A) -> Self {
        Self {
            raw: raw,
            generation: RefCell::new(0),
        }
    }

    pub fn generation(&self) -> u32 {
        *self.generation.borrow()
    }

    pub unsafe fn reset(&self) {
        *self.generation.borrow_mut() += 1;
        self.raw.reset();
    }
}

pub struct SGenAllocation<'a, T, A: TMemAllocator> {
    raw: T,
    generation: u32,
    temp_allocator: &'a SGenAllocator<A>,
}

impl<'a, T, A: TMemAllocator> SGenAllocation<'a, T, A> {
    pub fn valid(&self) -> bool {
        self.generation == self.temp_allocator.generation()
    }

    pub unsafe fn unwrap(&self) -> &T {
        if !self.valid() {
            panic!("Kept a GenAllocation around after reset!");
        }

        &self.raw
    }

    pub unsafe fn unwrap_mut(&mut self) -> &mut T {
        if !self.valid() {
            panic!("Kept a GenAllocation around after reset!");
        }

        &mut self.raw
    }
}

/*
impl<'a, 'b, T> IntoIterator for &'a SMemVec<'b, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b, T> IntoIterator for &'a mut SMemVec<'b, T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
*/

/* NOTES

I would really like to support some sort of temporary allocator that is reset at a frame, with
guarded access, but I don't think that's possible to do in a way that creates a generic allocation
result without one of:
a. unsafety
b. huge performance overhead

Maybe I can wrap allocations from it in some sort of TempAlloc struct?

I NEED TO WRITE BOTH VERSIONS FIRST, then find commonalities
*/

#[test]
fn test_basic() {
    let allocator = SSystemAllocator {};

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    vec.push(21);
    assert_eq!(vec[0], 33);
    assert_eq!(vec[1], 21);
    assert_eq!(vec.len(), 2);

    vec.push(9);
    assert_eq!(vec[0], 33);
    assert_eq!(vec[1], 21);
    assert_eq!(vec[2], 9);
    assert_eq!(vec.len(), 3);
}

#[test]
fn test_multiple_allocations() {
    let allocator = SSystemAllocator {};

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&allocator, 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_iter() {
    let allocator = SSystemAllocator {};

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    vec.push(0);
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);

    for (i, v) in vec.iter().enumerate() {
        assert_eq!(i as u32, *v);
    }
}

#[test]
fn test_genned() {
    let sys_allocator = SSystemAllocator {};
    let lin_allocator = SLinearAllocator::new(&sys_allocator, 1024, 8).unwrap();
    let gen_allocator = SGenAllocator::new(lin_allocator);

    let mut vec = SMemVec::<u32>::new_genned(&gen_allocator, 5, 0).unwrap();
    {
        let internal = unsafe { vec.unwrap() };
        assert_eq!(internal.len(), 0);
        assert_eq!(internal.capacity(), 5);
    }

    {
        let internal = unsafe { vec.unwrap_mut() };

        internal.push(33);
        assert_eq!(internal[0], 33);
        assert_eq!(internal.len(), 1);
    }
}

#[test]
#[should_panic]
fn test_genned_should_panic() {
    let sys_allocator = SSystemAllocator {};
    let lin_allocator = SLinearAllocator::new(&sys_allocator, 1024, 8).unwrap();
    let gen_allocator = SGenAllocator::new(lin_allocator);

    let mut vec = SMemVec::<u32>::new_genned(&gen_allocator, 5, 0).unwrap();
    {
        let internal = unsafe { vec.unwrap() };
        assert_eq!(internal.len(), 0);
        assert_eq!(internal.capacity(), 5);
    }

    unsafe { gen_allocator.reset() };

    {
        let internal = unsafe { vec.unwrap_mut() };

        internal.push(21);
        assert_eq!(internal[0], 33);
        assert_eq!(internal[1], 21);
        assert_eq!(internal.len(), 2);
    }
}

#[test]
fn test_stack_allocator() {
    let stack_allocator = SStackAllocator::new(&SYSTEM_ALLOCATOR, 1024, 8).unwrap();

    let mut vec = SMemVec::<u32>::new(&stack_allocator, 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&stack_allocator, 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_slice() {
    let allocator = SSystemAllocator {};

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    vec.push(33);
    vec.push(333);

    let vec_slice = &vec[..];
    assert_eq!(vec_slice[0], 33);
    assert_eq!(vec_slice[1], 333);
}