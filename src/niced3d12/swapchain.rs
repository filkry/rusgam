use super::*;

use arrayvec::ArrayVec;

pub struct SSwapChain {
    raw: t12::SSwapChain,

    pub buffercount: u32,
    pub backbuffers: ArrayVec<[SResource; 4]>,
}

impl SSwapChain {
    pub fn new_from_raw(raw: t12::SSwapChain, buffercount: u32) -> Self {
        assert!(buffercount <= 4);
        Self {
            raw: raw,
            buffercount: buffercount,
            backbuffers: ArrayVec::new(),
        }
    }

    pub fn raw(&self) -> &t12::SSwapChain {
        &self.raw
    }

    pub fn current_backbuffer_index(&self) -> usize {
        self.raw.currentbackbufferindex()
    }

    pub fn present(&mut self, sync_interval: u32, flags: u32) -> Result<(), &'static str> {
        self.raw.present(sync_interval, flags)
    }

    pub fn get_desc(&self) -> Result<t12::SSwapChainDesc, &'static str> {
        self.raw.getdesc()
    }

    pub fn resize_buffers(
        &mut self,
        buffercount: u32,
        width: u32,
        height: u32,
        olddesc: &t12::SSwapChainDesc,
    ) -> Result<(), &'static str> {
        self.raw.resizebuffers(buffercount, width, height, olddesc)
    }
}
