use super::*;

use std::mem;

struct SLinearUploadBufferAllocation<T> {
    cpu_mem: *mut T,
    gpu_mem: t12::SGPUVirtualAddress,
    generation: u64,
}

struct SLinearUploadBufferPage {
    page_size: usize,
    base_cpu_mem: *mut u8,
    base_gpu_mem: t12::SGPUVirtualAddress,
    first_free_byte_offset: usize,
    resource: SResource,
}

impl SLinearUploadBufferPage {
    fn new(device: &SDevice, page_size: usize) -> Result<Self, &'static str> {
        let resource = device.create_committed_buffer_resource(
            t12::EHeapType::Upload,
            t12::EHeapFlags::ENone,
            t12::SResourceFlags::none(),
            t12::EResourceStates::GenericRead,
            1,
            page_size,
        )?;

        Ok(Self{
            page_size: page_size,
            base_cpu_mem: ,
            base_gpu_mem: resource.raw().getgpuvirtualaddress(),
            first_free_byte_offset: 0,
            resource: resource,
        })
    }

    fn has_space<T>(&self, alignment: usize) -> bool {

    }

    fn allocate<T>(
        &mut self,
        alignment : usize,
    ) -> Result<SLinearUploadBufferAllocation<T>, &'static str> {

    }

    fn reset(&mut self) {

    }
}

struct SLinearUploadBuffer<'a> {
    page_size: usize,
    generation: u64,

    page_pool: Vec<SLinearUploadBufferPage>,
    cur_page: usize,

    device: &'a SDevice,
}

impl<'a> SLinearUploadBuffer<'a> {
    pub fn new() -> Self {
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn allocate<T>(
        &mut self,
        alignment : usize,
    ) -> Result<SLinearUploadBufferAllocation<T>, &'static str> {
        if mem::size_of::<T>() > self.page_size {
            return Err("Requested allocation larger than page size.");
        }

        if !self.page_pool[self.cur_page].has_space::<T>(alignment) {
            self.page_pool.push(SLinearUploadBufferPage::new());
            self.cur_page += 1
        }

        self.page_pool[self.cur_page].allocate::<T>(alignment)
    }

    pub fn reset(&mut self) {
        self.cur_page = 0;
        for page in &self.page_pool {
            page.reset();
        }
    }
}