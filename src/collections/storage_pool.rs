use allocate::{SAllocatorRef};

use super::pool::{TIndexGen, SPool, SPoolHandle};

// -- pool of storage for Ts. not every entry may be valid, and musn't always be initialized
pub struct SStoragePool<T, I: TIndexGen, G: TIndexGen> {
    pool: SPool<Option<T>, I, G>,
}

pub struct SStoragePoolIterator<'a, T, I: TIndexGen, G: TIndexGen> {
    pool: &'a SStoragePool<T, I, G>,
    next_idx: I,
}

impl<T, I: TIndexGen, G: TIndexGen> SStoragePool<T, I, G> {
    pub fn create(allocator: &SAllocatorRef, max: I) -> Self {
        Self {
            pool: SPool::<Option<T>, I, G>::create_default(allocator, max),
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
        Ok(int.as_ref())
    }

    pub fn get_by_index_mut(&mut self, index: I) -> Result<Option<&mut T>, &'static str> {
        let int = self.pool.get_by_index_mut(index)?;
        Ok(int.as_mut())
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

impl<'a, T, I: TIndexGen, G: TIndexGen> Iterator for SStoragePoolIterator<'a, T, I, G> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let pool_max = self.pool.max();
        while self.next_idx < pool_max {
            let cur = self.pool.get_by_index(self.next_idx).expect("loop bounded by pool_max");
            self.next_idx += I::ONE;
            if cur.is_some() {
                return cur;
            }
        }

        return None;
    }
}

impl<'a, T, I: TIndexGen, G: TIndexGen> IntoIterator for &'a SStoragePool<T, I, G> {
    type Item = &'a T;
    type IntoIter = SStoragePoolIterator<'a, T, I, G>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter{
            pool: self,
            next_idx: I::ZERO,
        }
    }
}
