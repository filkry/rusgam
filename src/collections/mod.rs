#![allow(dead_code)]

use std::collections::VecDeque;
//use std::cell::{RefCell, Ref, RefMut};

pub mod freelistallocator;

pub trait TIndexGen : PartialEq + PartialOrd + Copy + std::ops::Add + std::ops::AddAssign {
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    fn to_usize(&self) -> usize;
    fn from_usize(v: usize) -> Self;
}

impl TIndexGen for u16 {
    const MAX: u16 = std::u16::MAX;
    const ZERO: u16 = 0;
    const ONE: u16 = 1;

    fn to_usize(&self) -> usize {
        *self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}
impl TIndexGen for u32 {
    const MAX: u32 = std::u32::MAX;
    const ZERO: u32 = 0;
    const ONE: u32 = 1;

    fn to_usize(&self) -> usize {
        *self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}
impl TIndexGen for u64 {
    const MAX: u64 = std::u64::MAX;
    const ZERO: u64 = 0;
    const ONE: u64 = 1;

    fn to_usize(&self) -> usize {
        *self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SPoolHandle<I, G>
where I: TIndexGen, G: TIndexGen
{
    index: I,
    generation: G,
}

impl<I: TIndexGen, G: TIndexGen> SPoolHandle<I, G> {
    pub fn valid(&self) -> bool {
        self.index != I::MAX && self.generation != G::MAX
    }

    pub fn invalidate(&mut self) {
        *self = Default::default();
    }

    pub fn index(&self) -> I {
        self.index
    }

    pub fn generation(&self) -> G {
        self.generation
    }
}

impl<I: TIndexGen, G: TIndexGen> Default for SPoolHandle<I, G> {
    fn default() -> Self {
        SPoolHandle {
            index: I::MAX,
            generation: G::MAX,
        }
    }
}

// -- container of Ts, all of which must be initialized at all times. Meant for re-usable slots
// -- that don't need to be re-initialized
pub struct SPool<T, I: TIndexGen, G: TIndexGen> {
    // -- $$$FRK(TODO): make this into a string, only in debug builds?
    buffer: Vec<T>,
    generations: Vec<G>,
    max: I,
    freelist: VecDeque<I>,
}

impl<T, I: TIndexGen, G: TIndexGen> SPool<T, I, G> {
    // -- $$$FRK(TODO): I'd like to make these IDs either really smart, or just random
    pub fn create<F>(_id: u64, max: I, init_func: F) -> Self
    where
        F: Fn() -> T,
    {
        let mut result = Self {
            buffer: Vec::with_capacity(max.to_usize()),
            generations: Vec::with_capacity(max.to_usize()),
            max: max,
            freelist: VecDeque::new(),
        };

        result.buffer.resize_with(max.to_usize(), init_func);
        result.generations.resize(max.to_usize(), G::ZERO);

        for i in 0..max.to_usize() {
            result.freelist.push_back(I::from_usize(i));
        }

        result
    }

    pub fn create_from_vec(_id: u64, max: I, contents: Vec<T>) -> Self {
        let mut result = Self {
            buffer: contents,
            generations: Vec::with_capacity(max.to_usize()),
            max: max,
            freelist: VecDeque::new(),
        };

        result.generations.resize(max.to_usize(), G::ZERO);

        for i in 0..max.to_usize() {
            result.freelist.push_back(I::from_usize(i));
        }

        result
    }

    pub fn max(&self) -> I {
        self.max
    }

    pub fn used(&self) -> usize {
        (self.max.to_usize()) - self.free_count()
    }

    pub fn full(&self) -> bool {
        self.freelist.is_empty()
    }

    pub fn free_count(&self) -> usize {
        self.freelist.len()
    }

    pub fn alloc(&mut self) -> Result<SPoolHandle<I, G>, &'static str> {
        match self.freelist.pop_front() {
            Some(newidx) => {
                let idx = newidx.to_usize();
                Ok(SPoolHandle {
                    index: newidx,
                    generation: self.generations[idx],
                })
            }
            None => Err("Cannot alloc from full SPool."),
        }
    }

    pub fn free(&mut self, handle: SPoolHandle<I, G>) {
        if handle.valid() {
            let idx = handle.index.to_usize();
            if self.generations[idx] == handle.generation {
                self.generations[idx] += G::ONE;
                self.freelist.push_back(handle.index);
            }
        }
    }

    pub fn get(&self, handle: SPoolHandle<I, G>) -> Result<&T, &'static str> {
        let idx = handle.index.to_usize();
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            self.get_by_index(handle.index)
        } else {
            Err("Invalid, out of bounds, or stale handle.")
        }
    }

    pub fn get_mut(&mut self, handle: SPoolHandle<I, G>) -> Result<&mut T, &'static str> {
        let idx = handle.index.to_usize();
        if handle.valid() && handle.index < self.max && handle.generation == self.generations[idx] {
            self.getmutbyindex(handle.index)
        } else {
            Err("Invalid, out of bounds, or stale handle.")
        }
    }

    pub unsafe fn get_unchecked(&self, handle: SPoolHandle<I, G>) -> &T {
        &self.buffer[handle.index.to_usize()]
    }

    pub unsafe fn get_mut_unchecked(&mut self, handle: SPoolHandle<I, G>) -> &mut T {
        &mut self.buffer[handle.index.to_usize()]
    }

    fn get_by_index(&self, index: I) -> Result<&T, &'static str> {
        if index < self.max {
            Ok(&self.buffer[index.to_usize()])
        } else {
            Err("Out of bounds index")
        }
    }

    fn getmutbyindex(&mut self, index: I) -> Result<&mut T, &'static str> {
        if index < self.max {
            Ok(&mut self.buffer[index.to_usize()])
        } else {
            Err("Out of bounds index")
        }
    }

    pub fn handle_for_index(&self, index: I) -> Result<SPoolHandle<I, G>, &'static str> {
        if index < self.max {
            Ok(SPoolHandle {
                index: index,
                generation: self.generations[index.to_usize()],
            })
        } else {
            Err("Out of bounds index")
        }
    }
}

impl<T: Clone, I: TIndexGen, G: TIndexGen> SPool<T, I, G> {
    pub fn create_from_val(_id: u64, max: I, default_val: T) -> Self {
        let mut result = Self {
            buffer: Vec::new(),
            generations: Vec::new(),
            max: max,
            freelist: VecDeque::new(),
        };

        result.buffer.resize(max.to_usize(), default_val);
        result.generations.resize(max.to_usize(), G::ZERO);

        for i in 0..max.to_usize() {
            result.freelist.push_back(I::from_usize(i));
        }

        result
    }
}

impl<T: Default, I: TIndexGen, G: TIndexGen> SPool<T, I, G> {
    pub fn create_default(id: u64, max: I) -> Self {
        Self::create(id, max, Default::default)
    }
}

// -- pool of storage for Ts. not every entry may be valid, and musn't always be initialized
pub struct SStoragePool<T, I: TIndexGen, G: TIndexGen> {
    pool: SPool<Option<T>, I, G>, // -- $$$FRK(TODO): this could be unitialized mem that we use unsafety to construct/destruct in
}

impl<T, I: TIndexGen, G: TIndexGen> SStoragePool<T, I, G> {
    pub fn create(id: u64, max: I) -> Self {
        Self {
            pool: SPool::<Option<T>, I, G>::create_default(id, max),
        }
    }

    pub fn max(&self) -> I {
        self.pool.max()
    }

    pub fn used(&self) -> usize {
        self.pool.used()
    }

    pub fn handle_for_index(&self, index: I) -> Result<SPoolHandle<I, G>, &'static str> {
        self.pool.handle_for_index(index)
    }

    pub fn get_by_index(&self, index: I) -> Result<Option<&T>, &'static str> {
        let int = self.pool.get_by_index(index)?;
        match int {
            Some(a) => Ok(Some(&a)),
            None => Ok(None),
        }
    }

    pub fn insert_val(&mut self, val: T) -> Result<SPoolHandle<I, G>, &'static str> {
        let handle = self.pool.alloc()?;
        let data: &mut Option<T> = self.pool.get_mut(handle).unwrap();
        *data = Some(val);
        Ok(handle)
    }

    pub fn get(&self, handle: SPoolHandle<I, G>) -> Result<&T, &'static str> {
        let option = self.pool.get(handle)?;
        match option {
            Some(val) => Ok(&val),
            None => Err("nothing in handle"),
        }
    }

    pub fn get_mut(&mut self, handle: SPoolHandle<I, G>) -> Result<&mut T, &'static str> {
        let option = self.pool.get_mut(handle)?;
        match option {
            Some(ref mut val) => Ok(val),
            None => Err("nothing in handle"),
        }
    }

    pub fn free(&mut self, handle: SPoolHandle<I, G>) {
        let option = self.pool.get_mut(handle).unwrap();
        *option = None;
        self.pool.free(handle);
    }

    pub fn clear(&mut self) {
        let mut i = I::ZERO;
        while i < self.max() {
            let handle = self.pool.handle_for_index(i).expect("should only fail if i >= max");
            self.free(handle);
            i += I::ONE;
        }
    }
}

impl<T: Clone, I: TIndexGen, G: TIndexGen> SStoragePool<T, I, G> {
    pub fn insert_ref(&mut self, val: &T) -> Result<SPoolHandle<I, G>, &'static str> {
        let handle = self.pool.alloc()?;
        let data: &mut Option<T> = self.pool.get_mut(handle).unwrap();
        *data = Some(val.clone());
        Ok(handle)
    }
}

// -- $$$FRK(TODO): rewrite for new pool
/*
#[cfg(test)]
mod tests {
    use super::*; // imports the names from the non-test mod scope

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
}
*/
