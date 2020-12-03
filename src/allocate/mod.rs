#![allow(dead_code)]

//use std::iter::IntoIterator;
use std::mem::size_of;
use std::rc::{Rc, Weak};

pub mod memqueue;
pub mod system_allocator;
pub mod linear_allocator;
pub mod stack_allocator;

// -- $$$FRK(TODO): move queue into collections
pub use self::memqueue::*;
pub use self::system_allocator::*;
pub use self::linear_allocator::*;
pub use self::stack_allocator::*;

#[allow(non_snake_case)]
pub fn SYSTEM_ALLOCATOR() -> SAllocatorRef {
    SAllocator::new_system().as_ref()
}

thread_local! {
    pub static STACK_ALLOCATOR : SAllocator =
        SAllocator::new(SStackAllocator::new(SYSTEM_ALLOCATOR(), 4 * 1024 * 1024, 8).unwrap());
}

enum EAllocator {
    System,
    MemAllocator(Rc<dyn TMemAllocator>),
}

pub struct SAllocator {
    allocator: EAllocator,
}

#[derive(Clone)]
enum EAllocatorRef{
    System,
    MemAllocator(Weak<dyn TMemAllocator>),
}

#[derive(Clone)]
pub struct SAllocatorRef {
    allocator: EAllocatorRef,
}

pub struct SSystemAllocatorRef {}

pub trait TMemAllocator {
    // -- things implementing TMemAllocator should rely on internal mutability, since their
    // -- allocations will have a reference to them
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str>;
    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<(*mut u8, usize), &'static str>;

    // -- unsafe because it doesn't consume the SMem
    fn free(&self, existing_allocation: SMem) -> Result<(), &'static str>;
    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str>;

    fn reset(&self);
}

impl SAllocator {
    pub fn new<T: 'static + TMemAllocator>(allocator: T) -> Self {
        Self{
            allocator: EAllocator::MemAllocator(Rc::new(allocator)),
        }
    }

    pub fn new_system() -> Self {
        Self {
            allocator: EAllocator::System,
        }
    }

    pub fn as_ref(&self) -> SAllocatorRef {
        match &self.allocator {
            EAllocator::System => SAllocatorRef {
                allocator: EAllocatorRef::System,
            },
            EAllocator::MemAllocator(mem_allocator) => SAllocatorRef {
                allocator: EAllocatorRef::MemAllocator(Rc::downgrade(mem_allocator)),
            }
        }
    }

    pub fn reset(&self) {
        match &self.allocator {
            EAllocator::System => {
                // -- do nothing
            },
            EAllocator::MemAllocator(mem_allocator) => {
                mem_allocator.reset()
            }
        }
    }
}

impl SAllocatorRef {
    pub fn alloc(&self, size: usize, align: usize) -> Result<SMem, &'static str> {
        let (ptr, actual_size) = match &self.allocator {
            EAllocatorRef::System => {
                SSystemAllocator{}.alloc(size, align)?
            },
            EAllocatorRef::MemAllocator(mem_allocator) => {
                mem_allocator.upgrade()
                   .expect("trying to allocate from dropped allocator")
                   .alloc(size, align)?
            }
        };

        Ok(SMem{
            data: ptr,
            size: actual_size,
            alignment: align,
            allocator: self.clone(),
        })
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        match &self.allocator {
            EAllocatorRef::System => {
                SSystemAllocator{}.free_unsafe(existing_allocation)?
            },
            EAllocatorRef::MemAllocator(mem_allocator) => {
                mem_allocator.upgrade()
                    .expect("trying to allocate from dropped allocator")
                    .free_unsafe(existing_allocation)?
            }
        };

        Ok(())
    }
}

pub struct SMem {
    data: *mut u8,
    size: usize,
    alignment: usize,
    allocator: SAllocatorRef,
}

impl SMem {
    pub unsafe fn as_ref_typed<T>(&self) -> &T {
        assert!(self.size >= size_of::<T>());
        assert!(!self.data.is_null());
        (self.data as *const T).as_ref().expect("asserted on null above")
    }

    pub unsafe fn as_mut_typed<T>(&mut self) -> &mut T {
        assert!(self.size >= size_of::<T>());
        assert!(!self.data.is_null());
        (self.data as *mut T).as_mut().expect("asserted on null above")
    }

    pub unsafe fn data(&self) -> *mut u8 {
        self.data
    }

    fn invalidate(&mut self) {
        self.data = std::ptr::null_mut();
        self.size = 0;
        self.alignment = 0;
    }
}

impl Drop for SMem {
    fn drop(&mut self) {
        let allocator = self.allocator.clone();
        unsafe { allocator.free_unsafe(self).unwrap() };
    }
}

/*
pub struct SMemT<T> {
    mem: SMem,
    phantom: std::marker::PhantomData<T>,
}

impl<T> SMemT<T> {
    pub fn new(
        allocator: SAllocatorRef,
        mut value: T,
    ) -> Result<Self, &'static str> {
        let num_bytes = size_of::<T>();

        let mem = allocator.alloc(num_bytes, 8)?;

        let mut result = Ok(Self {
            mem,
            phantom: std::marker::PhantomData,
        })?;

        std::mem::swap(&mut value, result.deref_mut());
        std::mem::forget(value);

        Ok(result)
    }

    pub unsafe fn into_raw(mut self) -> SMem {
        let mut result = SMem {
            data: std::ptr::null_mut(),
            size: 0,
            alignment: 0,
            allocator: self.mem.allocator,
        };

        std::mem::swap(&mut result, &mut self.mem);
        std::mem::forget(self);

        result
    }

    pub unsafe fn from_raw(mem: SMem) -> Self {
        assert!(mem.size > size_of::<T>());
        Self {
            mem,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for SMemT<T> {
    fn drop(&mut self) {
        panic!("not implemented");
    }
}

impl<T> Deref for SMemT<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            (self.mem.data as *const T).as_ref().unwrap()
        }
    }
}

impl<T> DerefMut for SMemT<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            (self.mem.data as *mut T).as_mut().unwrap()
        }
    }
}
*/

#[test]
fn test_basic() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    vec.push(21);
    assert_eq!(vec[0], 33);
    assert_eq!(vec[1], 21);
    assert_eq!(vec.len(), 2);

    vec.push(9);
    assert_eq!(vec[0], 33);
    assert_eq!(vec[1], 21);
    assert_eq!(vec[2], 9);
    assert_eq!(vec.len(), 3);
}

#[test]
fn test_multiple_allocations() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&allocator, 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_iter() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    vec.push(0);
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);

    for (i, v) in vec.iter().enumerate() {
        assert_eq!(i as u32, *v);
    }
}

#[test]
fn test_drop() {
    let allocator = SYSTEM_ALLOCATOR();
    let refcount = RefCell::<i64>::new(0);

    struct SRefCounter<'a> {
        refcount: &'a RefCell::<i64>,
    }

    impl<'a> SRefCounter<'a> {
        pub fn new(refcount: &'a RefCell::<i64>) -> Self {
            *refcount.borrow_mut().deref_mut() += 1;

            Self {
                refcount,
            }
        }
    }

    impl<'a> Drop for SRefCounter<'a> {
        fn drop(&mut self) {
            *self.refcount.borrow_mut().deref_mut() -= 1;
        }
    }

    let mut vec = SMemVec::<SRefCounter>::new(&allocator, 5, 0).unwrap();

    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));
    vec.push(SRefCounter::new(&refcount));

    vec.clear();

    assert_eq!(*refcount.borrow().deref(), 0 as i64);
}

#[test]
fn test_linear_allocator() {
    let linear_allocator = SAllocator::new(SLinearAllocator::new(SYSTEM_ALLOCATOR(), 1024, 8).unwrap());

    let mut vec = SMemVec::<u32>::new(&linear_allocator.as_ref(), 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&linear_allocator.as_ref(), 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_stack_allocator() {
    let stack_allocator = SAllocator::new(SStackAllocator::new(SYSTEM_ALLOCATOR(), 1024, 8).unwrap());

    let mut vec = SMemVec::<u32>::new(&stack_allocator.as_ref(), 5, 0).unwrap();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 5);

    vec.push(33);
    assert_eq!(vec[0], 33);
    assert_eq!(vec.len(), 1);

    let mut vec2 = SMemVec::<u32>::new(&stack_allocator.as_ref(), 15, 0).unwrap();
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec2.capacity(), 15);

    vec2.push(333);
    assert_eq!(vec2[0], 333);
    assert_eq!(vec2.len(), 1);
}

#[test]
fn test_slice() {
    let allocator = SYSTEM_ALLOCATOR();

    let mut vec = SMemVec::<u32>::new(&allocator, 5, 0).unwrap();
    vec.push(33);
    vec.push(333);

    let vec_slice = &vec[..];
    assert_eq!(vec_slice[0], 33);
    assert_eq!(vec_slice[1], 333);
}
