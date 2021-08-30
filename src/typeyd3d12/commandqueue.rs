use arrayvec::{ArrayVec};

use super::*;

pub struct SCommandQueueDesc {
    raw: win::D3D12_COMMAND_QUEUE_DESC,
}

impl SCommandQueueDesc {
    pub fn cqtype(&self) -> ECommandListType {
        ECommandListType::new_from_d3dtype(self.raw.Type)
    }
}

#[derive(Clone)]
pub struct SCommandQueue {
    queue: win::ID3D12CommandQueue,
}

impl SCommandQueue {
    pub unsafe fn new_from_raw(raw: win::ID3D12CommandQueue) -> Self {
        Self { queue: raw }
    }

    pub unsafe fn raw(&self) -> &win::ID3D12CommandQueue {
        &self.queue
    }

    pub fn getdesc(&self) -> SCommandQueueDesc {
        SCommandQueueDesc {
            raw: unsafe { self.queue.GetDesc() },
        }
    }

    pub fn signal(&self, fence: &SFence, val: u64) -> Result<u64, &'static str> {
        let hn = unsafe { self.queue.Signal(fence.raw(), val) };

        returnerrifwinerror!(hn, "Could not push signal.");

        Ok(val)
    }

    // -- $$$FRK(TODO): support listS
    pub unsafe fn execute_command_lists(&self, lists: &[&mut SCommandList]) {
        let mut raw_lists = ArrayVec::<[Option::<win::ID3D12CommandList>; 12]>::new();
        for list in lists {
            raw_lists.push(Some(win::ID3D12CommandList::from(list.raw())));
        }

        self.queue
            .ExecuteCommandLists(raw_lists.len() as u32, &mut raw_lists[0]);
    }

    pub fn wait(&self, fence: &SFence, value: u64) -> Result<(), &'static str> {
        let hn = unsafe { self.queue.Wait(fence.raw(), value) };
        returnerrifwinerror!(hn, "Could not wait.");

        Ok(())
    }
}
