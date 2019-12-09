use super::*;

use collections::freelistallocator;

pub struct SDescriptorAllocatorAllocation {
    allocation: freelistallocator::manager::SAllocation,
    base_handle: t12::SCPUDescriptorHandle,
}

pub struct SDescriptorAllocator {
    descriptor_heap: SDescriptorHeap,
    heap_base_handle: t12::SCPUDescriptorHandle,
    allocator: freelistallocator::manager::SManager,
}

impl SDescriptorAllocator {
    pub fn new(
        device: &SDevice,
        num_descriptors: usize,
        descriptor_type: t12::EDescriptorHeapType
    ) -> Result<Self, &'static str> {

    }

    pub fn alloc(
        &mut self,
        num_descriptors: usize,
    ) -> Result<SDescriptorAllocatorAllocation, &'static str> {

    }

    pub fn free_on_signal(
        &mut self,
        allocation: SDescriptorAllocatorAllocation,
        signal: u64,
    ) {

    }

    pub fn signal(
        &mut self,
        signal: u64,
    ) {

    }
}