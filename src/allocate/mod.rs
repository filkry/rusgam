#![allow(dead_code)]

//use std::iter::IntoIterator;
use std::cell::RefCell;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::rc::{Rc, Weak};

use utils::align_up;

pub mod memqueue;

pub use self::memqueue::*;

#[allow(non_snake_case)]
pub fn SYSTEM_ALLOCATOR() -> SAllocatorRef {
    SAllocator::new_system().as_ref()
}

thread_local! {
    pub static STACK_ALLOCATOR : SAllocator =
        SAllocator::new(SStackAllocator::new(SYSTEM_ALLOCATOR(), 4 * 1024 * 1024, 8).unwrap());
}

enum EAllocator {
    System,
    MemAllocator(Rc<dyn TMemAllocator>),
}

pub struct SAllocator {
    allocator: EAllocator,
}

#[derive(Clone)]
enum EAllocatorRef{
    System,
    MemAllocator(Weak<dyn TMemAllocator>),
}

#[derive(Clone)]
pub struct SAllocatorRef {
    allocator: EAllocatorRef,
}

pub struct SSystemAllocatorRef {}

pub trait TMemAllocator {
    // -- things implementing TMemAllocator should rely on internal mutability, since their
    // -- allocations will have a reference to them
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str>;
    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<(*mut u8, usize), &'static str>;

    // -- unsafe because it doesn't consume the SMem
    fn free(&self, existing_allocation: SMem) -> Result<(), &'static str>;
    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str>;

    fn reset(&self);
}

impl SAllocator {
    pub fn new<T: 'static + TMemAllocator>(allocator: T) -> Self {
        Self{
            allocator: EAllocator::MemAllocator(Rc::new(allocator)),
        }
    }

    pub fn new_system() -> Self {
        Self {
            allocator: EAllocator::System,
        }
    }

    pub fn as_ref(&self) -> SAllocatorRef {
        match &self.allocator {
            EAllocator::System => SAllocatorRef {
                allocator: EAllocatorRef::System,
            },
            EAllocator::MemAllocator(mem_allocator) => SAllocatorRef {
                allocator: EAllocatorRef::MemAllocator(Rc::downgrade(mem_allocator)),
            }
        }
    }

    pub fn reset(&self) {
        match &self.allocator {
            EAllocator::System => {
                // -- do nothing
            },
            EAllocator::MemAllocator(mem_allocator) => {
                mem_allocator.reset()
            }
        }
    }
}

impl SAllocatorRef {
    pub fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str> {
        let (ptr, actual_size) = match &self.allocator {
            EAllocatorRef::System => {
                SSystemAllocator{}.alloc(size, align)?
            },
            EAllocatorRef::MemAllocator(mem_allocator) => {
                mem_allocator.upgrade()
                   .expect("trying to allocate from dropped allocator")
                   .alloc(size, align)?
            }
        };

        Ok(SMem{
            data: ptr,
            size: actual_size,
            alignment: align,
            allocator: self.clone(),
        })
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        match &self.allocator {
            EAllocatorRef::System => {
                SSystemAllocator{}.free_unsafe(existing_allocation)?
            },
            EAllocatorRef::MemAllocator(mem_allocator) => {
                mem_allocator.upgrade()
                    .expect("trying to allocate from dropped allocator")
                    .free_unsafe(existing_allocation)?
            }
        };

        Ok(())
    }
}

// -- $$$FRK(TODO): break allocators out in to their own files

pub struct SSystemAllocator {}

impl TMemAllocator for SSystemAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str> {
        let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
        let data = unsafe { std::alloc::alloc(layout) as *mut u8 };

        if data == std::ptr::null_mut() {
            return Err("failed to allocate");
        }

        Ok((data, size))
    }

    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<(*mut u8, usize), &'static str> {
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

        Ok((data, new_size))
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let layout = std::alloc::Layout::from_size_align(
            existing_allocation.size,
            existing_allocation.alignment,
        )
        .unwrap();

        std::alloc::dealloc(existing_allocation.data, layout);

        existing_allocation.data = std::ptr::null_mut();
        existing_allocation.size = 0;

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    fn reset(&self) {}
}

struct SLinearAllocatorAllocHeader {
    magic_number: u64,
    size: usize,
    freed: bool,
}

struct SLinearAllocatorData {
    raw: SMem,
    cur_offset: usize,
    allow_realloc: bool,
    held_allocations: usize,
}

pub struct SLinearAllocator {
    data: RefCell<SLinearAllocatorData>,
}

impl SLinearAllocator {
    pub fn new(
        parent: SAllocatorRef,
        size: usize,
        align: usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            data: RefCell::new(SLinearAllocatorData {
                raw: parent.alloc(size, align)?,
                cur_offset: 0,
                allow_realloc: false,
                held_allocations: 0,
            }),
        })
    }
}

impl SLinearAllocator {
    const HEADER_MAGIC_NUM : u64 = 0xf67d1a6399bb2139;
}

impl TMemAllocator for SLinearAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str> {
        let mut data = self.data.borrow_mut();

        if (data.raw.data as usize) % align != 0 {
            panic!("Currently don't support different alignments.");
        }

        let aligned_offset = align_up(data.cur_offset, align);
        let aligned_header_size = align_up(size_of::<SLinearAllocatorAllocHeader>(), align);
        let aligned_size = align_up(size, align);

        if (aligned_offset + aligned_header_size + aligned_size) > data.raw.size {
            return Err("Out of memory");
        }

        let header = unsafe { data.raw.data.add(aligned_offset) as *mut SLinearAllocatorAllocHeader};
        let result = unsafe { data.raw.data.add(aligned_offset + aligned_header_size) };

        unsafe {
            (*header).magic_number = Self::HEADER_MAGIC_NUM;
            (*header).size = aligned_size;
            (*header).freed = false;
        }

        data.cur_offset = aligned_offset + aligned_header_size + aligned_size;

        data.held_allocations += 1;

        Ok((result, aligned_size))
    }

    fn realloc(&self, _existing_allocation: SMem, _new_size: usize) -> Result<(*mut u8, usize), &'static str> {
        let data = self.data.borrow_mut();

        if data.allow_realloc {
            panic!("Not implemented.")
        }

        Err("Does not allow realloc.")
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let mut data = self.data.borrow_mut();

        let aligned_header_size = align_up(
            size_of::<SLinearAllocatorAllocHeader>(),
            existing_allocation.alignment,
        );

        let header = existing_allocation.data.sub(aligned_header_size) as *mut SLinearAllocatorAllocHeader;
        assert!((*header).magic_number == Self::HEADER_MAGIC_NUM);
        assert!((*header).size == existing_allocation.size);
        assert!((*header).freed == false);

        (*header).freed = true;

        assert!(data.held_allocations > 0);
        data.held_allocations -= 1;

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    fn reset(&self) {
        let mut data = self.data.borrow_mut();
        assert!(data.held_allocations == 0);

        data.cur_offset = 0;
    }
}

impl Drop for SLinearAllocator {
    fn drop(&mut self) {
        let data = self.data.borrow();
        assert!(data.held_allocations == 0);
    }
}

struct SStackAllocatorData {
    raw: SMem,
    top_offset: usize,
}

pub struct SStackAllocator {
    data: RefCell<SStackAllocatorData>,
}

impl SStackAllocator {
    pub fn new(
        parent: SAllocatorRef,
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
impl TMemAllocator for SStackAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str> {
        let mut data = self.data.borrow_mut();

        if (data.raw.data as usize) % align != 0 {
            panic!("Currently don't support different alignments.");
        }

        let aligned_offset = align_up(data.top_offset, align);
        let aligned_size = align_up(size, align);

        if (aligned_offset + aligned_size) > data.raw.size {
            return Err("Out of memory");
        }

        let result =  unsafe { data.raw.data.add(aligned_offset) };

        data.top_offset = aligned_offset + aligned_size;

        Ok((result, aligned_size))
    }

    fn realloc(&self, _existing_allocation: SMem, _new_size: usize) -> Result<(*mut u8, usize), &'static str> {
        panic!("Cannot re-alloc in stack allocator.")
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let mut data = self.data.borrow_mut();

        let ea_top = existing_allocation.data.add(existing_allocation.size);
        let self_top = data.raw.data.add(data.top_offset);
        if ea_top != self_top {
            println!("{:?}, {:?}", ea_top, self_top);
            panic!("Trying to free from the stack array, but not the top.");
        }

        data.top_offset = (existing_allocation.data as usize) - (data.raw.data as usize);
        existing_allocation.invalidate();

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    fn reset(&self) {
        panic!("Stack allocator does not handle reset!");
    }
}

pub struct SMem {
    data: *mut u8,
    size: usize,
    alignment: usize,
    allocator: SAllocatorRef,
}

impl SMem {
    pub unsafe fn as_ref_typed<T>(&self) -> &T {
        assert!(self.size >= size_of::<T>());
        assert!(!self.data.is_null());
        (self.data as *const T).as_ref().expect("asserted on null above")
    }

    pub unsafe fn as_mut_typed<T>(&mut self) -> &mut T {
        assert!(self.size >= size_of::<T>());
        assert!(!self.data.is_null());
        (self.data as *mut T).as_mut().expect("asserted on null above")
    }

    fn invalidate(&mut self) {
        self.data = std::ptr::null_mut();
        self.size = 0;
        self.alignment = 0;
    }
}

impl Drop for SMem {
    fn drop(&mut self) {
        let allocator = self.allocator.clone();
        unsafe { allocator.free_unsafe(self).unwrap() };
    }
}

/*
pub struct SMemT<T> {
    mem: SMem,
    phantom: std::marker::PhantomData<T>,
}

impl<T> SMemT<T> {
    pub fn new(
        allocator: SAllocatorRef,
        mut value: T,
    ) -> Result<Self, &'static str> {
        let num_bytes = size_of::<T>();

        let mem = allocator.alloc(num_bytes, 8)?;

        let mut result = Ok(Self {
            mem,
            phantom: std::marker::PhantomData,
        })?;

        std::mem::swap(&mut value, result.deref_mut());
        std::mem::forget(value);

        Ok(result)
    }

    pub unsafe fn into_raw(mut self) -> SMem {
        let mut result = SMem {
            data: std::ptr::null_mut(),
            size: 0,
            alignment: 0,
            allocator: self.mem.allocator,
        };

        std::mem::swap(&mut result, &mut self.mem);
        std::mem::forget(self);

        result
    }

    pub unsafe fn from_raw(mem: SMem) -> Self {
        assert!(mem.size > size_of::<T>());
        Self {
            mem,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for SMemT<T> {
    fn drop(&mut self) {
        panic!("not implemented");
    }
}

impl<T> Deref for SMemT<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            (self.mem.data as *const T).as_ref().unwrap()
        }
    }
}

impl<T> DerefMut for SMemT<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            (self.mem.data as *mut T).as_mut().unwrap()
        }
    }
}
*/

pub struct SMemVec<T> {
    mem: SMem,
    len: usize,
    capacity: usize,
    grow_capacity: usize,

    phantom: std::marker::PhantomData<T>,
}

impl<T> SMemVec<T> {
    pub fn new(
        allocator: &SAllocatorRef,
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

    pub fn new_copy_slice(
        allocator: &SAllocatorRef,
        slice: &[T],
    ) -> Result<Self, &'static str> {
        let initial_capacity = slice.len();
        let grow_capacity = 0;

        let num_bytes = initial_capacity * size_of::<T>();

        let mut result = Self {
            mem: allocator.alloc(num_bytes, 8)?,
            len: initial_capacity,
            capacity: initial_capacity,
            grow_capacity: grow_capacity,
            phantom: std::marker::PhantomData,
        };

        unsafe { std::ptr::copy_nonoverlapping(slice.as_ptr(), result.data_mut(), initial_capacity) };

        Ok(result)
    }

    unsafe fn data(&self) -> *const T {
        self.mem.data as *const T
    }

    unsafe fn data_mut(&mut self) -> *mut T {
        self.mem.data as *mut T
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data_mut(), self.len) }
    }

    pub fn push(&mut self, mut value: T) {
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
        std::mem::swap(&mut value, &mut self[idx]);
        std::mem::forget(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let mut replacement = unsafe { std::mem::MaybeUninit::<T>::zeroed().assume_init() };
        let last_idx = self.len() - 1;
        std::mem::swap(&mut self[last_idx], &mut replacement);

        self.len -= 1;
        return Some(replacement);
    }

    pub fn clear(&mut self) {
        for item in self.as_mut_slice() {
            unsafe {
                let mut replacement = std::mem::MaybeUninit::<T>::zeroed().assume_init();
                std::mem::swap(item, &mut replacement);
            }
        }

        self.len = 0;
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        let last_idx = self.len() - 1;
        self.as_mut_slice().swap(index, last_idx);

        // -- swap zeroes into the last value and pull out the result
        let mut result = unsafe { std::mem::MaybeUninit::<T>::zeroed().assume_init() };
        std::mem::swap(&mut self[last_idx], &mut result);
        self.len -= 1;

        result
    }
}

impl<T: Clone> SMemVec<T> {
    pub fn push_all(&mut self, val: T) {
        while self.remaining_capacity() > 0 {
            self.push(val.clone());
        }
    }
}

impl<T: Default> SMemVec<T> {
    pub fn push_all_default(&mut self) {
        while self.remaining_capacity() > 0 {
            self.push(Default::default());
        }
    }
}

impl<T> Drop for SMemVec<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> Deref for SMemVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for SMemVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> Index<I> for SMemVec<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        //Index::index(&**self, index)
        Index::index(self.deref(), index) // relying on Index implementation of &[T]
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> IndexMut<I> for SMemVec<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        //IndexMut::index_mut(&mut **self, index)
        IndexMut::index_mut(self.deref_mut(), index)
    }
}

impl SMemVec<u8> {
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice()) }
    }
}

impl std::io::Write for SMemVec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {

        for ch in buf {
            self.push(*ch);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

#[test]
fn test_basic() {
    let allocator = SYSTEM_ALLOCATOR();

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
    let allocator = SYSTEM_ALLOCATOR();

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
    let allocator = SYSTEM_ALLOCATOR();

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
fn test_drop() {
    let allocator = SYSTEM_ALLOCATOR();
    let refcount = RefCell::<i64>::new(0);

    struct SRefCounter<'a> {
        refcount: &'a RefCell::<i64>,
    }

    impl<'a> SRefCounter<'a> {
        pub fn new(refcount: &'a RefCell::<i64>) -> Self {
            *refcount.borrow_mut().deref_mut() += 1;

            Self {
                refcount,
            }
        }
    }

    impl<'a> Drop for SRefCounter<'a> {
        fn drop(&mut self) {
            *self.refcount.borrow_mut().deref_mut() -= 1;
        }
    }

    let mut vec = SMemVec::<SRefCounter>::new(&allocator, 5, 0).unwrap();

    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));

    vec.clear();

    assert_eq!(*refcount.borrow().deref(), 0 as i64);
}

#[test]
fn test_linear_allocator() {
    let linear_allocator = SAllocator::new(SLinearAllocator::new(SYSTEM_ALLOCATOR(), 1024, 8).unwrap());

    let mut vec = SMemVec::<u32>::new(&linear_allocator.as_ref(), 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&linear_allocator.as_ref(), 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_stack_allocator() {
    let stack_allocator = SAllocator::new(SStackAllocator::new(SYSTEM_ALLOCATOR(), 1024, 8).unwrap());

    let mut vec = SMemVec::<u32>::new(&stack_allocator.as_ref(), 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&stack_allocator.as_ref(), 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_slice() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    vec.push(33);
    vec.push(333);

    let vec_slice = &vec[..];
    assert_eq!(vec_slice[0], 33);
    assert_eq!(vec_slice[1], 333);
}
