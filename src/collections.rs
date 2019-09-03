#![allow(dead_code)]

use std::alloc::*;
use std::ops::{Index, IndexMut};
use std::collections::VecDeque;

pub struct STypedBuffer<T: Copy> {
    // -- $$$FRK(TODO): support allocator other than system heap
    // -- $$$FRK(TODO): while I'm NOT supporting different allocators, is this easier as a Box-d Array?
    // Or even possibly we should have a way to create Box-d slices from the custom allocators, and
    // then get all the built-in functionality of slice
    count: u32,
    buffer: *mut T,
}

impl<T: Copy> STypedBuffer<T> {
    fn create(count: u32) -> STypedBuffer<T> {
        STypedBuffer::<T> {
            count: count,
            buffer: std::ptr::null_mut(),
        }
    }

    fn alloc(&mut self) {
        let eightbytealign = 8;
        let layoutres = Layout::from_size_align(
            (self.count as usize) * std::mem::size_of::<T>(),
            eightbytealign,
        );
        let layout = layoutres.unwrap(); // $$$FRK(TODO): handle
        self.buffer = unsafe { alloc(layout) as *mut T };
    }
}

impl<T: Copy> Index<isize> for STypedBuffer<T> {
    type Output = T;
    fn index<'a>(&'a self, index: isize) -> &'a T {
        assert!(self.buffer != std::ptr::null_mut());
        assert!(
            index >= 0 && index < (self.count as isize),
            "Trying to get invalid index into STypedBuffer."
        );
        // -- $$$FRK(TODO): handle unwrap?
        unsafe {
            return self.buffer.offset(index).as_ref().unwrap();
        }
    }
}

impl<T: Copy> IndexMut<isize> for STypedBuffer<T> {
    fn index_mut<'a>(&'a mut self, index: isize) -> &'a mut T {
        assert!(self.buffer != std::ptr::null_mut());
        assert!(
            index >= 0 && index < (self.count as isize),
            "Trying to get invalid index into STypedBuffer."
        );
        // -- $$$FRK(TODO): handle unwrap?
        unsafe {
            return self.buffer.offset(index).as_mut().unwrap();
        }
    }
}

impl<T: Copy> Drop for STypedBuffer<T> {
    fn drop(&mut self) {
        let eightbytealign = 8;
        let layoutres = Layout::from_size_align(
            (self.count as usize) * std::mem::size_of::<T>(),
            eightbytealign,
        );
        let layout = layoutres.unwrap(); // $$$FRK(TODO): handle

        unsafe { dealloc(self.buffer as *mut u8, layout) };
    }
}

pub struct SFixedQueue<T: Copy> {
    nextpushidx: u32,
    nextpopidx: u32,
    curcount: u32,
    buffer: STypedBuffer<T>,
}

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
        } else {
            None
        }
    }
}

pub struct SPool<T: Default + Copy> {
    buffer: Vec<T>,
    generations: Vec<u16>,
    max: u16,
    freelist: VecDeque<u16>,
    setup: bool,
}

impl<T: Default + Copy> Default for SPool<T> {
    fn default() -> Self {
        SPool {
            buffer: Vec::new(),
            generations: Vec::new(),
            max: 0,
            freelist: VecDeque::new(),
            setup: false,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SPoolHandle {
    index: u16,
    generation: u16,
}

impl SPoolHandle {
    fn valid(&self) -> bool {
        self.index != std::u16::MAX && self.generation != std::u16::MAX
    }
}

impl Default for SPoolHandle {
    fn default() -> Self {
        SPoolHandle {
            index: std::u16::MAX,
            generation: std::u16::MAX,
        }
    }
}

impl<T: Default + Copy> SPool<T> {
    pub fn setup(&mut self, max: u16) {
        assert_eq!(self.setup, false);

        self.buffer.resize(max as usize, Default::default());
        self.generations.resize(max as usize, 0);
        self.max = max;

        for i in 0..max {
            self.freelist.push_back(i);
        }

        self.setup = true;
    }

    pub fn full(&self) -> bool {
        !self.freelist.is_empty()
    }

    pub fn pushval(&mut self, val: T) -> Result<SPoolHandle, &'static str> {
        self.push(&val)
    }

    pub fn push(&mut self, val: &T) -> Result<SPoolHandle, &'static str> {
        match self.freelist.pop_front() {
            Some(newidx) => {
                let idx = newidx as usize;
                self.buffer[idx] = *val;
                Ok(SPoolHandle{
                    index: newidx,
                    generation: self.generations[idx]
                })
            }
            None => Err("Cannot push to full SPool.")
        }
    }

    pub fn pop(&mut self, handle: SPoolHandle) {
        if handle.valid() {
            let idx = handle.index as usize;
            if self.generations[idx] == handle.generation {
                self.buffer[idx] = Default::default();
                self.generations[idx] += 1;
                self.freelist.push_back(handle.index);
            }
        }
    }

    pub fn get(&self, handle: SPoolHandle) -> Result<&T, &'static str> {
        let idx = handle.index as usize;
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            Ok(&self.buffer[idx])
        }
        else {
            Err("Invalid, out of bounds, or stale handle.")
        }
    }

    pub fn getmut(&mut self, handle: SPoolHandle) -> Result<&mut T, &'static str> {
        let idx = handle.index as usize;
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            Ok(&mut self.buffer[idx])
        }
        else {
            Err("Invalid, out of bounds, or stale handle.")
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

    #[test]
    fn test_basicpool_nonmut() {
        let mut p : SPool<u64> = Default::default();
        p.setup(10);

        let ahandle = p.pushval(234).unwrap();
        let bhandle = p.pushval(023913).unwrap();

        assert_eq!(*p.get(bhandle).unwrap(), 023913);
        assert_eq!(*p.get(ahandle).unwrap(), 234);

        *p.getmut(ahandle).unwrap() = 432;
        *p.getmut(bhandle).unwrap() = 9293231;

        assert_eq!(*p.get(bhandle).unwrap(), 9293231);
        assert_eq!(*p.get(ahandle).unwrap(), 432);
    }
}
