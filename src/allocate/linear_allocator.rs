use std::cell::RefCell;
use std::mem::size_of;

use utils::align_up;

use super::{SMem, TMemAllocator, SAllocatorRef};

struct SLinearAllocatorAllocHeader {
    magic_number: u64,
    size: usize,
    freed: bool,
}

struct SLinearAllocatorData {
    raw: SMem,
    cur_offset: usize,
    allow_realloc: bool,
    held_allocations: usize,
}

pub struct SLinearAllocator {
    data: RefCell<SLinearAllocatorData>,
}

impl SLinearAllocator {
    pub fn new(
        parent: SAllocatorRef,
        size: usize,
        align: usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            data: RefCell::new(SLinearAllocatorData {
                raw: parent.alloc(size, align)?,
                cur_offset: 0,
                allow_realloc: false,
                held_allocations: 0,
            }),
        })
    }
}

impl SLinearAllocator {
    const HEADER_MAGIC_NUM : u64 = 0xf67d1a6399bb2139;
}

impl TMemAllocator for SLinearAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str> {
        let mut data = self.data.borrow_mut();

        if (data.raw.data as usize) % align != 0 {
            panic!("Currently don't support different alignments.");
        }

        let aligned_offset = align_up(data.cur_offset, align);
        let aligned_header_size = align_up(size_of::<SLinearAllocatorAllocHeader>(), align);
        let aligned_size = align_up(size, align);

        if (aligned_offset + aligned_header_size + aligned_size) > data.raw.size {
            return Err("Out of memory");
        }

        let header = unsafe { data.raw.data.add(aligned_offset) as *mut SLinearAllocatorAllocHeader};
        let result = unsafe { data.raw.data.add(aligned_offset + aligned_header_size) };

        unsafe {
            (*header).magic_number = Self::HEADER_MAGIC_NUM;
            (*header).size = aligned_size;
            (*header).freed = false;
        }

        data.cur_offset = aligned_offset + aligned_header_size + aligned_size;

        data.held_allocations += 1;

        Ok((result, aligned_size))
    }

    fn realloc(&self, _existing_allocation: SMem, _new_size: usize) -> Result<(*mut u8, usize), &'static str> {
        let data = self.data.borrow_mut();

        if data.allow_realloc {
            panic!("Not implemented.")
        }

        Err("Does not allow realloc.")
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let mut data = self.data.borrow_mut();

        let aligned_header_size = align_up(
            size_of::<SLinearAllocatorAllocHeader>(),
            existing_allocation.alignment,
        );

        let header = existing_allocation.data.sub(aligned_header_size) as *mut SLinearAllocatorAllocHeader;
        assert!((*header).magic_number == Self::HEADER_MAGIC_NUM);
        assert!((*header).size == existing_allocation.size);
        assert!((*header).freed == false);

        (*header).freed = true;

        assert!(data.held_allocations > 0);
        data.held_allocations -= 1;

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    fn reset(&self) {
        let mut data = self.data.borrow_mut();
        assert!(data.held_allocations == 0);

        data.cur_offset = 0;
    }
}

impl Drop for SLinearAllocator {
    fn drop(&mut self) {
        let data = self.data.borrow();
        assert!(data.held_allocations == 0);
    }
}


