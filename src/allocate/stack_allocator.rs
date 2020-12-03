use std::cell::RefCell;

use utils::align_up;

use super::{SMem, TMemAllocator, SAllocatorRef};

struct SStackAllocatorData {
    raw: SMem,
    top_offset: usize,
}

pub struct SStackAllocator {
    data: RefCell<SStackAllocatorData>,
}

impl SStackAllocator {
    pub fn new(
        parent: SAllocatorRef,
        size: usize,
        align: usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            data: RefCell::new(SStackAllocatorData {
                raw: parent.alloc(size, align)?,
                top_offset: 0,
            }),
        })
    }
}

impl TMemAllocator for SStackAllocator {
    fn alloc(&self, size: usize, align: usize) -> Result<(*mut u8, usize), &'static str> {
        let mut data = self.data.borrow_mut();

        if (data.raw.data as usize) % align != 0 {
            panic!("Currently don't support different alignments.");
        }

        let aligned_offset = align_up(data.top_offset, align);
        let aligned_size = align_up(size, align);

        if (aligned_offset + aligned_size) > data.raw.size {
            return Err("Out of memory");
        }

        let result =  unsafe { data.raw.data.add(aligned_offset) };

        data.top_offset = aligned_offset + aligned_size;

        Ok((result, aligned_size))
    }

    fn realloc(&self, _existing_allocation: SMem, _new_size: usize) -> Result<(*mut u8, usize), &'static str> {
        panic!("Cannot re-alloc in stack allocator.")
    }

    unsafe fn free_unsafe(&self, existing_allocation: &mut SMem) -> Result<(), &'static str> {
        let mut data = self.data.borrow_mut();

        let ea_top = existing_allocation.data.add(existing_allocation.size);
        let self_top = data.raw.data.add(data.top_offset);
        if ea_top != self_top {
            println!("{:?}, {:?}", ea_top, self_top);
            panic!("Trying to free from the stack array, but not the top.");
        }

        data.top_offset = (existing_allocation.data as usize) - (data.raw.data as usize);
        existing_allocation.invalidate();

        Ok(())
    }

    fn free(&self, mut existing_allocation: SMem) -> Result<(), &'static str> {
        unsafe { self.free_unsafe(&mut existing_allocation) }
    }

    fn reset(&self) {
        panic!("Stack allocator does not handle reset!");
    }
}


