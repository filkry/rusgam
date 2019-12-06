use super::*;

pub struct SCommandAllocator {
    raw: t12::SCommandAllocator,
}

impl SCommandAllocator {
    pub unsafe fn new_from_raw(raw: t12::SCommandAllocator) -> Self {
        Self {
            raw: raw,
        }
    }

    pub unsafe fn raw(&self) -> &t12::SCommandAllocator {
        &self.raw
    }

    pub fn reset(&mut self) {
        self.raw.reset();
    }
}

