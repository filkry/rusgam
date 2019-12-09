#![allow(dead_code)]

use std::collections::VecDeque;
//use std::cell::{RefCell, Ref, RefMut};

pub mod freelistallocator;

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
    where
        F: Fn() -> T,
    {
        let mut result = Self {
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

    pub fn create_from_vec(id: u64, max: u16, contents: Vec<T>) -> Self {
        let mut result = Self {
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
        let mut result = Self {
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
    pool: SPool<Option<T>>, // -- $$$FRK(TODO): this could be unitialized mem that we use unsafety to construct/destruct in
}

impl<T> SStoragePool<T> {
    pub fn create(id: u64, max: u16) -> Self {
        Self {
            pool: SPool::<Option<T>>::create_default(id, max),
        }
    }

    pub fn insert_val(&mut self, val: T) -> Result<SPoolHandle, &'static str> {
        let handle = self.pool.alloc()?;
        let data: &mut Option<T> = self.pool.get_mut(handle).unwrap();
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
