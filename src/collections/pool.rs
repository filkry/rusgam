use std::collections::VecDeque;

use crate::allocate::{SAllocatorRef};
use crate::collections::{SVec};

pub trait TIndexGen : PartialEq + PartialOrd + Copy + std::ops::Add + std::ops::AddAssign {
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    fn to_usize(&self) -> usize;
    fn from_usize(v: usize) -> Self;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SPoolHandle<I, G>
where I: TIndexGen, G: TIndexGen
{
    index: I,
    generation: G,
}

// -- container of Ts, all of which must be initialized at all times. Meant for re-usable slots
// -- that don't need to be re-initialized
pub struct SPool<T, I: TIndexGen, G: TIndexGen> {
    buffer: SVec<T>,
    generations: SVec<G>,
    max: I,
    freelist: VecDeque<I>,
}

impl<I: TIndexGen, G: TIndexGen> Default for SPoolHandle<I, G> {
    fn default() -> Self {
        SPoolHandle {
            index: I::MAX,
            generation: G::MAX,
        }
    }
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

impl<T, I: TIndexGen, G: TIndexGen> SPool<T, I, G> {
    pub fn create<F>(allocator: &SAllocatorRef, max: I, init_func: F) -> Self
    where
        F: Fn() -> T,
    {
        let mut result = Self {
            buffer: SVec::new(allocator, max.to_usize(), 0).expect("failed to allocate SVec"),
            generations: SVec::new(allocator, max.to_usize(), 0).expect("failed to allocate SVec"),
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

    pub fn create_from_vec(allocator: &SAllocatorRef, max: I, contents: SVec<T>) -> Self {
        let mut result = Self {
            buffer: contents,
            generations: SVec::new(allocator, max.to_usize(), 0).expect("failed to allocate SVec"),
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
            self.get_by_index_mut(handle.index)
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

    pub fn get_by_index(&self, index: I) -> Result<&T, &'static str> {
        if index < self.max {
            Ok(&self.buffer[index.to_usize()])
        } else {
            Err("Out of bounds index")
        }
    }

    pub fn get_by_index_mut(&mut self, index: I) -> Result<&mut T, &'static str> {
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
    pub fn create_from_val(allocator: &SAllocatorRef, max: I, default_val: T) -> Self {
        let mut result = Self {
            buffer: SVec::new(allocator, max.to_usize(), 0).expect("failed to allocate vec"),
            generations: SVec::new(allocator, max.to_usize(), 0).expect("failed to allocate vec"),
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
    pub fn create_default(allocator: &SAllocatorRef, max: I) -> Self {
        Self::create(allocator, max, Default::default)
    }
}
