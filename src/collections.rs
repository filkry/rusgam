#![allow(dead_code)]

use std::alloc::*;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};
//use std::cell::{RefCell, Ref, RefMut};

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

#[derive(Copy, Clone)]
pub struct SPoolHandle {
    poolid: u64,
    index: u16,
    generation: u16,
}

impl SPoolHandle {
    pub fn valid(&self) -> bool {
        self.index != std::u16::MAX && self.generation != std::u16::MAX
    }

    pub fn invalidate(&mut self) {
        *self = Default::default();
    }
}

impl Default for SPoolHandle {
    fn default() -> Self {
        SPoolHandle {
            poolid: std::u64::MAX,
            index: std::u16::MAX,
            generation: std::u16::MAX,
        }
    }
}

pub struct SPool<T> {
    // -- $$$FRK(TODO): make this into a string, only in debug builds?
    id: u64, // -- for making sure we have the right pool

    buffer: Vec<T>,
    generations: Vec<u16>,
    max: u16,
    freelist: VecDeque<u16>,
}

impl<T> SPool<T> {
    // -- $$$FRK(TODO): I'd like to make these IDs either really smart, or just random
    pub fn create<F>(id: u64, max: u16, init_func: F) -> Self
        where F: Fn() -> T,
    {
        let mut result = Self{
            id: id,
            buffer: Vec::with_capacity(max as usize),
            generations: Vec::with_capacity(max as usize),
            max: max,
            freelist: VecDeque::new(),
        };

        result.buffer.resize_with(max as usize, init_func);
        result.generations.resize(max as usize, 0);

        for i in 0..max {
            result.freelist.push_back(i);
        }

        result
    }

    pub fn create_from_vec<F>(id: u64, max: u16, contents: Vec<T>) -> Self
        where F: Fn() -> T,
    {
        let mut result = Self{
            id: id,
            buffer: contents,
            generations: Vec::with_capacity(max as usize),
            max: max,
            freelist: VecDeque::new(),
        };

        result.generations.resize(max as usize, 0);

        for i in 0..max {
            result.freelist.push_back(i);
        }

        result
    }

    pub fn full(&self) -> bool {
        self.freelist.is_empty()
    }

    pub fn alloc(&mut self) -> Result<SPoolHandle, &'static str> {
        match self.freelist.pop_front() {
            Some(newidx) => {
                let idx = newidx as usize;
                Ok(SPoolHandle {
                    index: newidx,
                    generation: self.generations[idx],
                    poolid: self.id,
                })
            }
            None => Err("Cannot alloc from full SPool."),
        }
    }

    pub fn free(&mut self, handle: SPoolHandle) {
        if handle.valid() {
            let idx = handle.index as usize;
            if self.generations[idx] == handle.generation {
                self.generations[idx] += 1;
                self.freelist.push_back(handle.index);
            }
        }
    }

    pub fn get(&self, handle: SPoolHandle) -> Result<&T, &'static str> {
        let idx = handle.index as usize;
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            self.getbyindex(handle.index)
        } else {
            Err("Invalid, out of bounds, or stale handle.")
        }
    }

    pub fn get_mut(&mut self, handle: SPoolHandle) -> Result<&mut T, &'static str> {
        let idx = handle.index as usize;
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            self.getmutbyindex(handle.index)
        } else {
            Err("Invalid, out of bounds, or stale handle.")
        }
    }

    fn getbyindex(&self, index: u16) -> Result<&T, &'static str> {
        if index < self.max {
            Ok(&self.buffer[index as usize])
        } else {
            Err("Out of bounds index")
        }
    }

    fn getmutbyindex(&mut self, index: u16) -> Result<&mut T, &'static str> {
        if index < self.max {
            Ok(&mut self.buffer[index as usize])
        } else {
            Err("Out of bounds index")
        }
    }

    pub fn handleforindex(&self, index: u16) -> Result<SPoolHandle, &'static str> {
        if index < self.max {
            Ok(SPoolHandle {
                index: index,
                generation: self.generations[index as usize],
                poolid: self.id,
            })
        } else {
            Err("Out of bounds index")
        }
    }
}

impl<T: Clone> SPool<T> {
    pub fn create_from_val(id: u64, max: u16, default_val: T) -> Self {
        let mut result = Self{
            id: id,
            buffer: Vec::new(),
            generations: Vec::new(),
            max: max,
            freelist: VecDeque::new(),
        };

        result.buffer.resize(max as usize, default_val);
        result.generations.resize(max as usize, 0);

        for i in 0..max {
            result.freelist.push_back(i);
        }

        result
    }

}

impl<T: Default> SPool<T> {
    pub fn create_default(id: u64, max: u16) -> Self {
        Self::create(id, max, Default::default)
    }
}

pub struct SStoragePool<T> {
    pool: SPool<Option<T>> // -- $$$FRK(TODO): this could be unitialized mem that we use unsafety to construct/destruct in
}

impl<T> SStoragePool<T> {
    pub fn create(id: u64, max: u16) -> Self {
        Self{
            pool: SPool::<Option<T>>::create_default(id, max),
        }
    }

    pub fn insert_val(&mut self, val: T) -> Result<SPoolHandle, &'static str> {
        let handle = self.pool.alloc()?;
        let data : &mut Option<T> = self.pool.get_mut(handle).unwrap();
        *data = Some(val);
        Ok(handle)
    }

    pub fn get(&self, handle: SPoolHandle) -> Result<&T, &'static str> {
        let option = self.pool.get(handle)?;
        match option {
            Some(val) => Ok(&val),
            None => Err("nothing in handle"),
        }
    }

    pub fn get_mut(&mut self, handle: SPoolHandle) -> Result<&mut T, &'static str> {
        let option = self.pool.get_mut(handle)?;
        match option {
            Some(ref mut val) => Ok(val),
            None => Err("nothing in handle"),
        }
    }

    pub fn free(&mut self, handle: SPoolHandle) {
        let option = self.pool.get_mut(handle).unwrap();
        *option = None;
        self.pool.free(handle);
    }
}

impl<T: Clone> SStoragePool<T> {
    pub fn insert_ref(&mut self, val: &T) -> Result<SPoolHandle, &'static str> {
        let handle = self.pool.alloc()?;
        let data : &mut Option<T> = self.pool.get_mut(handle).unwrap();
        *data = Some(val.clone());
        Ok(handle)
    }
}

/*
pub struct SRefCellPool<T: Default + Clone> {
    buffer: Vec<RefCell<T>>,
    generations: Vec<u16>,
    max: u16,
    freelist: VecDeque<u16>,
    setup: bool,
    defaultonfree: bool,
}

impl<T: Default + Clone> Default for SRefCellPool<T> {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            generations: Vec::new(),
            max: 0,
            freelist: VecDeque::new(),
            setup: false,
            defaultonfree: false,
        }
    }
}

impl<T: Default + Clone> SRefCellPool<T> {
    pub fn setup(&mut self, max: u16) {
        assert_eq!(self.setup, false);

        self.generations.resize(max as usize, 0);
        self.max = max;

        for i in 0..max {
            self.buffer.push(RefCell::new(Default::default()));
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
                *self.buffer[idx].borrow_mut() = val.clone();
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
                if self.defaultonfree {
                    *self.buffer[idx].borrow_mut() = Default::default();
                }
                self.generations[idx] += 1;
                self.freelist.push_back(handle.index);
            }
        }
    }

    pub fn get(&self, handle: SPoolHandle) -> Result<Ref<T>, &'static str> {
        let idx = handle.index as usize;
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            Ok(self.buffer[idx].borrow())
        }
        else {
            Err("Invalid, out of bounds, or stale handle.")
        }
    }

    pub fn getmut(&self, handle: SPoolHandle) -> Result<RefMut<T>, &'static str> {
        let idx = handle.index as usize;
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            Ok(self.buffer[idx].borrow_mut())
        }
        else {
            Err("Invalid, out of bounds, or stale handle.")
        }
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
    fn test_pool_basic() {
        let mut p: SPool<u64> = Default::default();
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

    /*
    #[test]
    fn test_refcellpool() {
        let mut p : SRefCellPool<u64> = Default::default();
        p.setup(10);

        let ahandle = p.pushval(234).unwrap();
        let bhandle = p.pushval(023913).unwrap();

        assert_eq!(*p.get(bhandle).unwrap(), 023913);
        assert_eq!(*p.get(ahandle).unwrap(), 234);

        *p.getmut(ahandle).unwrap() = 432;
        *p.getmut(bhandle).unwrap() = 9293231;

        assert_eq!(*p.get(bhandle).unwrap(), 9293231);
        assert_eq!(*p.get(ahandle).unwrap(), 432);

        let mut a = p.getmut(ahandle).unwrap();
        let mut b = p.getmut(bhandle).unwrap();

        *a = 34;
        *b = 12;
    }
    */
}
