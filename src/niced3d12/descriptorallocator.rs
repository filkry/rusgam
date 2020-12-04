use super::*;

use collections::freelistallocator;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct SDescriptorAllocatorAllocation {
    allocation: Option<freelistallocator::manager::SAllocation>,
    base_cpu_handle: t12::SCPUDescriptorHandle,
    base_gpu_handle: t12::SGPUDescriptorHandle,
    descriptor_size: usize,
    num_handles: usize,

    allocator: Weak<SDescriptorAllocator>,
}

impl SDescriptorAllocatorAllocation {
    // -- $$$FRK(FUTURE WORK): maybe this should work like the thread-local storage in rust, where you
    // -- have to pass a function, and a reference can't escape the scope of that function?
    pub fn cpu_descriptor(&self, idx: usize) -> t12::SCPUDescriptorHandle {
        self.allocation.as_ref().unwrap().validate();

        if idx >= self.num_handles {
            panic!("Index out of bounds!");
        }

        unsafe { self.base_cpu_handle.offset(idx * self.descriptor_size) }
    }

    pub fn gpu_descriptor(&self, idx: usize) -> t12::SGPUDescriptorHandle {
        self.allocation.as_ref().unwrap().validate();

        if idx >= self.num_handles {
            panic!("Index out of bounds!");
        }

        unsafe { self.base_gpu_handle.offset(idx * self.descriptor_size) }
    }
}

impl Drop for SDescriptorAllocatorAllocation {
    fn drop(&mut self) {
        if let Some(_a) = &self.allocation {
            self.allocator.upgrade().expect("allocator freed before allocation").free(self);
        }
    }
}

struct SDescriptorAllocatorPendingFree {
    allocation: freelistallocator::manager::SAllocation,
    signal: u64,
}

struct SDescriptorAllocatorInternal {
    descriptor_heap: SDescriptorHeap,
    allocator: freelistallocator::manager::SManager,

    pending_frees: Vec<SDescriptorAllocatorPendingFree>,
    last_signal: Option<u64>,
}

pub struct SDescriptorAllocator {
    descriptor_type: t12::EDescriptorHeapType,
    heap_base_handle: t12::SCPUDescriptorHandle,

    internal: RefCell<SDescriptorAllocatorInternal>, // this exists to ease internal mutability
}

impl SDescriptorAllocator {
    pub fn new(
        device: &SDevice,
        num_descriptors: usize,
        descriptor_type: t12::EDescriptorHeapType,
        flags: t12::SDescriptorHeapFlags,
    ) -> Result<Self, &'static str> {
        let desc = t12::SDescriptorHeapDesc {
            type_: descriptor_type,
            num_descriptors: num_descriptors,
            flags: flags,
        };

        let descriptor_heap = device.create_descriptor_heap(&desc)?;
        let heap_start = descriptor_heap.cpu_handle_heap_start();

        Ok(Self {
            descriptor_type: descriptor_type,
            heap_base_handle: heap_start,

            internal: RefCell::new(SDescriptorAllocatorInternal{
                descriptor_heap: descriptor_heap,
                allocator: freelistallocator::manager::SManager::new(num_descriptors),
                pending_frees: Vec::new(),
                last_signal: None,
            }),
        })
    }

    pub fn with_raw_heap<F>(&self, mut func: F)
        where F: FnMut(&SDescriptorHeap) -> () {
        func(&self.internal.borrow().descriptor_heap);
    }

    pub fn type_(&self) -> t12::EDescriptorHeapType {
        self.descriptor_type
    }

    pub fn free(&self, allocation: &mut SDescriptorAllocatorAllocation) {
        self.internal.borrow_mut().allocator.free(allocation.allocation.as_mut().unwrap());
    }

    pub fn free_on_signal(&self, mut allocation: SDescriptorAllocatorAllocation, signal: u64) {
        let mut internal = self.internal.borrow_mut();

        if let Some(s) = internal.last_signal {
            if signal <= s {
                internal.allocator.free(allocation.allocation.as_mut().unwrap());
                return;
            }
        }

        let pf = SDescriptorAllocatorPendingFree {
            allocation: allocation.allocation.take().unwrap(),
            signal: signal,
        };
        internal.pending_frees.push(pf);
    }

    pub fn signal(&self, signal: u64) {
        let mut internal = self.internal.borrow_mut();

        assert!(signal >= internal.last_signal.unwrap_or(0));

        let mut idx = 0;
        while idx < internal.pending_frees.len() {
            if internal.pending_frees[idx].signal <= signal {
                let mut allocation = internal.pending_frees.swap_remove(idx).allocation;
                internal.allocator.free(&mut allocation);
            } else {
                idx += 1;
            }
        }

        internal.last_signal = Some(signal);
    }
}

pub fn descriptor_alloc(
    allocator: &Rc<SDescriptorAllocator>,
    num_descriptors: usize,
) -> Result<SDescriptorAllocatorAllocation, &'static str> {
    let mut internal = allocator.internal.borrow_mut();

    let allocation = internal.allocator.alloc(num_descriptors, 1)?;
    let base_cpu_handle = internal.descriptor_heap.cpu_handle(allocation.start_offset())?;
    let base_gpu_handle = internal.descriptor_heap.gpu_handle(allocation.start_offset())?;

    Ok(SDescriptorAllocatorAllocation {
        allocation: Some(allocation),
        base_cpu_handle: base_cpu_handle,
        base_gpu_handle: base_gpu_handle,
        descriptor_size: internal.descriptor_heap.descriptorsize,
        num_handles: num_descriptors,

        allocator: Rc::downgrade(&allocator),
    })
}

impl Drop for SDescriptorAllocator {
    fn drop(&mut self) {
        // -- free any pending allocations
        self.signal(std::u64::MAX);
    }
}
