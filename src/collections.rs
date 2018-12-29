use std::alloc::*;
use std::ops::{Index, IndexMut};

#[allow(dead_code)]
pub struct STypedBuffer<T> {
    count: u32,
    buffer: *mut T,
}

#[allow(dead_code)]
impl<T> STypedBuffer<T> {
    fn create(count: u32) -> STypedBuffer<T> {
        STypedBuffer::<T> {
            count: count,
            buffer: std::ptr::null_mut(),
        }
    }

    fn alloc(&mut self) {
        let eightbytealign = 8;
        let layoutres = Layout::from_size_align((self.count as usize) * std::mem::size_of::<T>(), eightbytealign);
        let layout = layoutres.unwrap(); // $$$FRK(TODO): handle
        self.buffer = unsafe { alloc(layout) as *mut T };
    }
}

impl<T> Index<isize> for STypedBuffer<T> {
    type Output = T;
    fn index<'a>(&'a self, index: isize) -> &'a T {
        assert!(index >= 0 && index < (self.count as isize), "Trying to get invalid index into STypedBuffer.");
        // -- $$$FRK(TODO): handle unwrap?
        unsafe {
            return self.buffer.offset(index).as_ref().unwrap();
        }
    }
}

impl<T> IndexMut<isize> for STypedBuffer<T> {
    fn index_mut<'a>(&'a mut self, index: isize) -> &'a mut T {
        assert!(index >= 0 && index < (self.count as isize), "Trying to get invalid index into STypedBuffer.");
        // -- $$$FRK(TODO): handle unwrap?
        unsafe {
            return self.buffer.offset(index).as_mut().unwrap();
        }
    }
}

/*
#[allow(dead_code)]
pub struct SFixedQueue<T> {
    // -- $$$FRK(TODO): support allocator other than system heap
    max: u32,
    cur: u32,
    buffer: STypedBuffer<T>,
}

#[allow(dead_code)]
impl<T> SFixedQueue<T> {
    pub fn alloc(&mut self) {
        self.buffer.alloc();
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*; // imports the names from the non-test mod scope

    #[test]
    fn test_basictypedbuffer() {
        let mut tb = STypedBuffer::<i32>::create(10);
        tb.alloc();
        tb[5] = 10;
        assert_eq!(tb[5], 10);
    }
}
