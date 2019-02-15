use std::alloc::*;
use std::ops::{Index, IndexMut};

#[allow(dead_code)]
pub struct STypedBuffer<T: Copy> {
    // -- $$$FRK(TODO): support allocator other than system heap
    // -- $$$FRK(TODO): while I'm NOT supporting different allocators, is this easier as a Box-d Array?
    // Or even possibly we should have a way to create Box-d slices from the custom allocators, and
    // then get all the built-in functionality of slice
    count: u32,
    buffer: *mut T,
}

#[allow(dead_code)]
impl<T: Copy> STypedBuffer<T> {
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

impl<T: Copy> Index<isize> for STypedBuffer<T> {
    type Output = T;
    fn index<'a>(&'a self, index: isize) -> &'a T {
        assert!(self.buffer != std::ptr::null_mut());
        assert!(index >= 0 && index < (self.count as isize), "Trying to get invalid index into STypedBuffer.");
        // -- $$$FRK(TODO): handle unwrap?
        unsafe {
            return self.buffer.offset(index).as_ref().unwrap();
        }
    }
}

impl<T: Copy> IndexMut<isize> for STypedBuffer<T> {
    fn index_mut<'a>(&'a mut self, index: isize) -> &'a mut T {
        assert!(self.buffer != std::ptr::null_mut());
        assert!(index >= 0 && index < (self.count as isize), "Trying to get invalid index into STypedBuffer.");
        // -- $$$FRK(TODO): handle unwrap?
        unsafe {
            return self.buffer.offset(index).as_mut().unwrap();
        }
    }
}

impl<T: Copy> Drop for STypedBuffer<T> {
    fn drop(&mut self) {
        let eightbytealign = 8;
        let layoutres = Layout::from_size_align((self.count as usize) * std::mem::size_of::<T>(), eightbytealign);
        let layout = layoutres.unwrap(); // $$$FRK(TODO): handle

        unsafe { dealloc(self.buffer as *mut u8, layout) };
    }
}

#[allow(dead_code)]
pub struct SFixedQueue<T: Copy> {
    nextpushidx: u32,
    nextpopidx: u32,
    curcount: u32,
    buffer: STypedBuffer<T>,
}

#[allow(dead_code)]
impl<T: Copy> SFixedQueue<T> {
    pub fn create(max: u32) -> SFixedQueue<T> {
        SFixedQueue::<T> {
            nextpushidx: 0,
            nextpopidx: 0,
            curcount: 0,
            buffer: STypedBuffer::<T>::create(max),
        }
    }

    // $$$FRK(TODO): allocation must return an error that must be handled - can we bake this
    // into the lifetime somehow?
    pub fn alloc(&mut self) {
        self.buffer.alloc();
    }

    pub fn full(&mut self) -> bool {
        return self.curcount >= self.buffer.count;
    }

    pub fn empty(&mut self) -> bool {
        return self.curcount == 0;
    }

    pub fn push(&mut self, v: T) {
        if !self.full() {
            self.buffer[self.nextpushidx as isize] = v;
            self.nextpushidx = (self.nextpushidx + 1) % self.buffer.count;
            self.curcount += 1;
        }
    }

    pub fn pushref(&mut self, v: &T) {
        if !self.full() {
            self.buffer[self.nextpushidx as isize] = *v;
            self.nextpushidx = (self.nextpushidx + 1) % self.buffer.count;
            self.curcount += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if !self.empty() {
            let result: T = self.buffer[self.nextpopidx as isize];
            self.nextpopidx = (self.nextpopidx + 1) % self.buffer.count;
            self.curcount -= 1;
            Some(result)
        }
        else {
            None
        }
    }
}

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

    #[test]
    fn test_basicfixedqueue() {
        let mut q = SFixedQueue::<i32>::create(1);
        q.alloc();

        assert!(q.empty());
        q.push(1);
        assert!(!q.empty());
        assert!(q.full());
        assert_eq!(1, q.pop().unwrap());
        assert!(q.empty());
    }

    #[test]
    fn test_lessbasicfixedqueue() {
        let mut q = SFixedQueue::<i32>::create(3);
        q.alloc();

        assert!(q.empty());

        q.push(1);
        q.push(2);
        q.push(3);

        assert!(!q.empty());

        assert_eq!(1, q.pop().unwrap());
        assert_eq!(2, q.pop().unwrap());
        assert_eq!(3, q.pop().unwrap());

        assert!(q.empty());

        q.push(4);
        q.push(5);
        assert_eq!(4, q.pop().unwrap());
        assert_eq!(5, q.pop().unwrap());

        assert!(q.empty());
    }
}
