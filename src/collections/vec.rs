use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use allocate::{SMem, SAllocatorRef};

pub struct SVec<T> {
    mem: SMem,
    len: usize,
    capacity: usize,
    grow_capacity: usize,

    phantom: std::marker::PhantomData<T>,
}

impl<T> SVec<T> {
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
        self.mem.data() as *const T
    }

    unsafe fn data_mut(&mut self) -> *mut T {
        self.mem.data() as *mut T
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

impl<T: Clone> SVec<T> {
    pub fn push_all(&mut self, val: T) {
        while self.remaining_capacity() > 0 {
            self.push(val.clone());
        }
    }
}

impl<T: Default> SVec<T> {
    pub fn push_all_default(&mut self) {
        while self.remaining_capacity() > 0 {
            self.push(Default::default());
        }
    }
}

impl<T> Drop for SVec<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> Deref for SVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for SVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> Index<I> for SVec<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        //Index::index(&**self, index)
        Index::index(self.deref(), index) // relying on Index implementation of &[T]
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> IndexMut<I> for SVec<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        //IndexMut::index_mut(&mut **self, index)
        IndexMut::index_mut(self.deref_mut(), index)
    }
}

impl SVec<u8> {
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice()) }
    }
}

impl std::io::Write for SVec<u8> {
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


