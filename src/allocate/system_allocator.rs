use super::{SMem, TMemAllocator};

pub struct SSystemAllocator {}

impl TMemAllocator for SSystemAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str> {
        let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
        let data = unsafe { std::alloc::alloc(layout) as *mut u8 };

        if data == std::ptr::null_mut() {
            return Err("failed to allocate");
        }

        Ok((data, size))
    }

    fn realloc(&self, existing_allocation: SMem, new_size: usize) -> Result<(*mut u8, usize), &'static str> {
        let layout = std::alloc::Layout::from_size_align(
            existing_allocation.size,
            existing_allocation.alignment,
        )
        .unwrap();
        let data =
            unsafe { std::alloc::realloc(existing_allocation.data, layout, new_size) as *mut u8 };

        if data == std::ptr::null_mut() {
            // -- failed to re-alloc, free memory and run
            self.free(existing_allocation)?;
            return Err("failed to re-alloc");
        }

        Ok((data, new_size))
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let layout = std::alloc::Layout::from_size_align(
            existing_allocation.size,
            existing_allocation.alignment,
        )
        .unwrap();

        std::alloc::dealloc(existing_allocation.data, layout);

        existing_allocation.data = std::ptr::null_mut();
        existing_allocation.size = 0;

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    fn reset(&self) {}
}


