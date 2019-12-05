use super::*;

pub struct SCommandQueueDesc {
    raw: D3D12_COMMAND_QUEUE_DESC,
}

impl SCommandQueueDesc {
    pub fn cqtype(&self) -> ECommandListType {
        ECommandListType::new_from_d3dtype(self.raw.Type)
    }
}

#[derive(Clone)]
pub struct SCommandQueue {
    queue: ComPtr<ID3D12CommandQueue>,
}

impl SCommandQueue {
    pub unsafe fn new_from_raw(raw: ComPtr<ID3D12CommandQueue>) -> Self {
        Self {
            queue: raw,
        }
    }

    pub unsafe fn raw(&self) -> &ComPtr<ID3D12CommandQueue> {
        &self.queue
    }

    pub fn getdesc(&self) -> SCommandQueueDesc {
        SCommandQueueDesc {
            raw: unsafe { self.queue.GetDesc() },
        }
    }

    // -- $$$FRK(TODO): revisit this after I understand how I'm going to be using this fence
    pub fn signal(&self, fence: &SFence, val: u64) -> Result<u64, &'static str> {
        let hn = unsafe { self.queue.Signal(fence.raw().as_raw(), val) };

        returnerrifwinerror!(hn, "Could not push signal.");

        Ok(val)
    }

    // -- $$$FRK(TODO): support listS
    pub unsafe fn executecommandlist(&self, list: &SCommandList) {
        self.queue
            .ExecuteCommandLists(1, &(list.raw().as_raw() as *mut ID3D12CommandList));
    }
}
