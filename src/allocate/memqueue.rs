use safewindows;

use super::*;

pub struct SMemQueue<T> {
    mem: SMem,
    len: usize,
    capacity: usize,
    first: usize,
    last: usize,

    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> SMemQueue<T> {
    pub fn new(
        allocator: &SAllocatorRef,
        capacity: usize,
    ) -> Result<Self, &'static str> {
        let num_bytes = capacity * size_of::<T>();

        Ok(Self {
            mem: allocator.alloc(num_bytes, 8)?,
            len: 0,
            capacity: capacity,
            first: 0,
            last: 0,
            phantom: std::marker::PhantomData,
        })
    }

    fn data(&self) -> *mut T {
        self.mem.data as *mut T
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

    fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data(), self.capacity) }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data(), self.capacity) }
    }

    pub fn push_back(&mut self, mut value: T) {
        if self.len == self.capacity {
            break_assert!(false); // -- out of space
            return;
        }

        self.len += 1;
        self.last = (self.last + 1) % self.capacity;
        let idx = {
            if self.last == 0 {
                self.capacity - 1
            }
            else {
                self.last - 1
            }
        };
        std::mem::swap(&mut value, &mut self.as_mut_slice()[idx]);
        std::mem::forget(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        unsafe {
            let mut result = std::mem::MaybeUninit::<T>::zeroed().assume_init();
            let first_idx = self.first;
            std::mem::swap(&mut self.as_mut_slice()[first_idx], &mut result);

            self.len -= 1;
            self.first = (self.first + 1) % self.capacity;

            Some(result)
        }
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
}

#[test]
fn test_basic() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut q = SMemQueue::<u32>::new(&allocator, 5).unwrap();
    assert_eq!(q.len(), 0);
    assert_eq!(q.capacity(), 5);

    let mut push_test = |i: u32, expected_len: usize| {
        q.push_back(i);
        assert_eq!(q.len(), expected_len);
    };

    push_test(33, 1);
    push_test(21, 2);
    push_test(9, 3);
    push_test(18, 4);
    push_test(29, 5);

    let mut pop_test = |expected_i: u32, expected_len: usize| {
        let val = q.pop_front();
        assert!(val.is_some());
        assert_eq!(val.unwrap(), expected_i);
        assert_eq!(q.len(), expected_len);
    };

    pop_test(33, 4);
    pop_test(21, 3);
    pop_test(9, 2);
    pop_test(18, 1);
    pop_test(29, 0);

    let val = q.pop_front();
    assert!(val.is_none());
}

#[test]
fn test_ring() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut q = SMemQueue::<u32>::new(&allocator, 3).unwrap();
    assert_eq!(q.len(), 0);
    assert_eq!(q.capacity(), 3);

    let push_test = |q: &mut SMemQueue<u32>, i: u32, expected_len: usize| {
        q.push_back(i);
        assert_eq!(q.len(), expected_len);
    };

    let pop_test = |q: &mut SMemQueue<u32>, expected_i: u32, expected_len: usize| {
        let val = q.pop_front();
        assert!(val.is_some());
        assert_eq!(val.unwrap(), expected_i);
        assert_eq!(q.len(), expected_len);
    };

    push_test(&mut q, 33, 1);
    pop_test(&mut q, 33, 0);

    push_test(&mut q, 21, 1);
    push_test(&mut q, 9, 2);
    push_test(&mut q, 18, 3);
    pop_test(&mut q, 21, 2);
    pop_test(&mut q, 9, 1);
    pop_test(&mut q, 18, 0);

    push_test(&mut q, 29, 1);
    pop_test(&mut q, 29, 0);

    let val = q.pop_front();
    assert!(val.is_none());
}
