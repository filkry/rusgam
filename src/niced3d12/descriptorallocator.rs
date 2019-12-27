use super::*;

use collections::freelistallocator;

pub struct SDescriptorAllocatorAllocation {
    allocation: freelistallocator::manager::SAllocation,
    base_handle: t12::SCPUDescriptorHandle,
    num_handles: usize,
}

impl SDescriptorAllocatorAllocation {

    // -- $$$FRK(TODO): maybe this should work like the thread-local storage in rust, where you
    // -- have to pass a function, and a reference can't escape the scope of that function?
    pub fn descriptor(&self, idx: usize) -> t12::SCPUDescriptorHandle {
        if idx >= self.num_handles {
            panic!("Index out of bounds!");
        }

        unsafe { self.base_handle.offset(idx) }
    }
}

struct SDescriptorAllocatorPendingFree {
    allocation: freelistallocator::manager::SAllocation,
    signal: u64,
}

pub struct SDescriptorAllocator {
    descriptor_type: t12::EDescriptorHeapType,
    descriptor_heap: SDescriptorHeap,
    heap_base_handle: t12::SCPUDescriptorHandle,
    allocator: freelistallocator::manager::SManager,

    pending_frees: Vec<SDescriptorAllocatorPendingFree>,
    last_signal: Option<u64>,
}

impl SDescriptorAllocator {
    pub fn new(
        device: &SDevice,
        num_descriptors: usize,
        descriptor_type: t12::EDescriptorHeapType,
    ) -> Result<Self, &'static str> {
        let desc = t12::SDescriptorHeapDesc {
            type_: descriptor_type,
            num_descriptors: num_descriptors,
            flags: t12::SDescriptorHeapFlags::none(),
        };

        let descriptor_heap = device.create_descriptor_heap(&desc)?;
        let heap_start = descriptor_heap.cpu_handle_heap_start();

        Ok(Self {
            descriptor_type: descriptor_type,
            descriptor_heap: descriptor_heap,
            heap_base_handle: heap_start,
            allocator: freelistallocator::manager::SManager::new(num_descriptors),

            pending_frees: Vec::new(),
            last_signal: None,
        })
    }

    pub fn alloc(
        &mut self,
        num_descriptors: usize,
    ) -> Result<SDescriptorAllocatorAllocation, &'static str> {
        let allocation = self.allocator.alloc(num_descriptors, 1)?;
        let base_handle = self.descriptor_heap.cpu_handle(allocation.start_offset())?;

        Ok(SDescriptorAllocatorAllocation {
            allocation: allocation,
            base_handle: base_handle,
            num_handles: num_descriptors,
        })
    }

    pub fn free_on_signal(&mut self, allocation: SDescriptorAllocatorAllocation, signal: u64) {
        if let Some(s) = self.last_signal {
            if signal <= s {
                self.allocator.free(allocation.allocation);
                return;
            }
        }

        let pf = SDescriptorAllocatorPendingFree {
            allocation: allocation.allocation,
            signal: signal,
        };
        self.pending_frees.push(pf);
    }

    pub fn signal(&mut self, signal: u64) {
        assert!(signal >= self.last_signal.unwrap_or(0));

        let mut idx = 0;
        while idx < self.pending_frees.len() {
            if self.pending_frees[idx].signal <= signal {
                self.allocator
                    .free(self.pending_frees.swap_remove(idx).allocation);
            } else {
                idx += 1;
            }
        }

        self.last_signal = Some(signal);
    }
}
