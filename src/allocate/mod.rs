#![allow(dead_code)]

use std::mem::{size_of};
use std::ops::{Deref, DerefMut, Index, IndexMut};

pub trait TMemAllocator {
    // -- things implementing TMemAllocator should rely on internal mutability, since their
    // -- allocations will have a reference to them
    fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str>;
    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<SMem, &'static str>;
    fn free(&self, existing_allocation: SMem) -> Result<(), &'static str>;
}

pub struct SSystemAllocator {

}

impl TMemAllocator for SSystemAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str> {
        let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
        let data = unsafe { std::alloc::alloc(layout) as *mut u8};

        if data == std::ptr::null_mut() {
            return Err("failed to allocate");
        }

        Ok(SMem{
            data: data,
            size: size,
            alignment: align,
            allocator: self,
        })
    }

    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<SMem, &'static str> {
        let layout = std::alloc::Layout::from_size_align(existing_allocation.size, existing_allocation.alignment).unwrap();
        let data = unsafe { std::alloc::realloc(existing_allocation.data, layout, new_size) as *mut u8};

        if data == std::ptr::null_mut() {
            // -- failed to re-alloc, free memory and run
            self.free(existing_allocation)?;
            return Err("failed to re-alloc");
        }

        Ok(SMem{
            data: data,
            size: new_size,
            alignment: existing_allocation.alignment,
            allocator: self,
        })
    }

    fn free(&self, existing_allocation: SMem) -> Result<(), &'static str> {
        let layout = std::alloc::Layout::from_size_align(existing_allocation.size, existing_allocation.alignment).unwrap();
        unsafe { std::alloc::dealloc(existing_allocation.data, layout) };

        Ok(())
    }
}

pub struct SMem<'a> {
    data: *mut u8,
    size: usize,
    alignment: usize,
    allocator: &'a dyn TMemAllocator,
}

impl<'a> Drop for SMem<'a> {
    fn drop(&mut self) {
        let allocator = self.allocator;

        // -- this is the only safe place to copy an SMem, as we know we are dropping
        // -- it immediately. We do this so we can enforce move on free in TMemAllocator
        let copied = SMem{
            data: self.data,
            size: self.size,
            alignment: self.alignment,
            allocator: self.allocator,
        };

        allocator.free(copied).unwrap();
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
    pub fn new(allocator: &'a dyn TMemAllocator, initial_capacity: usize, grow_capacity: usize) -> Result<Self, &'static str> {
        let num_bytes = initial_capacity * size_of::<T>();

        Ok(Self {
            mem: allocator.alloc(num_bytes, 8)?,
            len: 0,
            capacity: initial_capacity,
            grow_capacity: grow_capacity,
            phantom: std::marker::PhantomData,
        })
    }

    fn data(&self) -> *mut T {
        self.mem.data as *mut T
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            if self.grow_capacity == 0 {
                assert!(false, "Out of space, not pushing.");
                return;
            }
            else {
                panic!("Grow not implemented!")
            }
        }

        self.len += 1;
        let idx = (self.len - 1) as isize;
        self[idx] = value;
    }
}

impl<'a, T> Deref for SMemVec<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.data(), self.len)
        }
    }
}

impl<'a, T> DerefMut for SMemVec<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data(), self.len)
        }
    }
}

impl<'a, T> Index<isize> for SMemVec<'a, T> {
    type Output = T;
    fn index(&self, index: isize) -> &T {
        if index < 0 || index >= (self.len() as isize) {
            panic!("Trying to get invalid index into SMemVec.");
        }

        unsafe {
            return self.data().offset(index).as_ref().unwrap();
        }
    }
}

impl<'a, T> IndexMut<isize> for SMemVec<'a, T> {
    fn index_mut(&mut self, index: isize) -> &mut T {
        if index < 0 || index >= (self.len() as isize) {
            panic!("Trying to get invalid index into SMemVec.");
        }

        unsafe {
            return self.data().offset(index).as_mut().unwrap();
        }
    }
}

/*
#[test]
fn test_basic() {
    let allocator = SManager::new(100);

    let allocation = allocator.alloc(1, 1).unwrap();
    assert_eq!(allocation.start_offset, 0);
    assert_eq!(allocation.size, 1);

    allocator.free(allocation);
    assert_eq!(allocator.free_chunks.len(), 1);
    assert_eq!(allocator.free_chunks[0].start_offset, 0);
    assert_eq!(allocator.free_chunks[0].size, 100);
}
*/
