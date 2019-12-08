use super::*;

use utils::align_up;

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

        Ok(Self {
            page_size: page_size,
            base_cpu_mem: unsafe { resource.raw().map(0, None)? },
            base_gpu_mem: resource.raw().getgpuvirtualaddress(),
            first_free_byte_offset: 0,
            resource: resource,
        })
    }

    fn has_space<T>(&self, alignment: usize) -> bool {
        let aligned_size = align_up(mem::size_of::<T>(), alignment);
        let aligned_offset = align_up(self.first_free_byte_offset, alignment);

        (aligned_offset + aligned_size) < self.page_size
    }

    fn allocate<T>(
        &mut self,
        alignment: usize,
        generation: u64,
    ) -> Result<SLinearUploadBufferAllocation<T>, &'static str> {
        if !self.has_space::<T>(alignment) {
            return Err("Not enough space to allocate from this page.");
        }

        let aligned_size = align_up(mem::size_of::<T>(), alignment);
        self.first_free_byte_offset = align_up(self.first_free_byte_offset, alignment);

        let ptr = unsafe { self.base_cpu_mem.add(self.first_free_byte_offset) };
        let t_ptr = ptr as *mut T;

        let result = SLinearUploadBufferAllocation::<T> {
            cpu_mem: t_ptr,
            gpu_mem: self.base_gpu_mem.add(self.first_free_byte_offset),
            generation: generation,
        };

        self.first_free_byte_offset += aligned_size;

        Ok(result)
    }

    fn reset(&mut self) {}
}

struct SLinearUploadBuffer<'a> {
    page_size: usize,
    generation: u64,

    page_pool: Vec<SLinearUploadBufferPage>,
    cur_page: usize,

    device: &'a SDevice,
}

impl<'a> SLinearUploadBuffer<'a> {
    pub fn new(device: &'a SDevice, page_size: usize) -> Result<Self, &'static str> {
        let mut result = SLinearUploadBuffer {
            page_size: page_size,
            generation: 0,
            page_pool: Vec::new(),
            cur_page: 0,
            device: device,
        };

        result
            .page_pool
            .push(SLinearUploadBufferPage::new(device, page_size)?);

        Ok(result)
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn allocate<T>(
        &mut self,
        alignment: usize,
    ) -> Result<SLinearUploadBufferAllocation<T>, &'static str> {
        if mem::size_of::<T>() > self.page_size {
            return Err("Requested allocation larger than page size.");
        }

        if !self.page_pool[self.cur_page].has_space::<T>(alignment) {
            if let None = self.page_pool.get(self.cur_page + 1) {
                self.page_pool
                    .push(SLinearUploadBufferPage::new(self.device, self.page_size)?);
            }
            self.cur_page += 1
        }

        self.page_pool[self.cur_page].allocate::<T>(alignment, self.generation)
    }

    pub fn reset(&mut self) {
        self.generation += 1;
        self.cur_page = 0;
        for page in &mut self.page_pool {
            page.reset();
        }
    }
}
