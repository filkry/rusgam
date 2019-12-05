use super::*;

pub struct SInfoQueue {
    infoqueue: ComPtr<ID3D12InfoQueue>,
}

impl SInfoQueue {
    pub unsafe fn new_from_raw(raw: ComPtr<ID3D12InfoQueue>) -> Self {
        Self {
            infoqueue: raw,
        }
    }

    pub fn setbreakonseverity(&self, id: D3D12_MESSAGE_ID, val: BOOL) {
        unsafe {
            self.infoqueue.SetBreakOnSeverity(id, val);
        }
    }

    pub fn pushstoragefilter(
        &self,
        filter: &mut D3D12_INFO_QUEUE_FILTER,
    ) -> Result<(), &'static str> {
        let hn = unsafe { self.infoqueue.PushStorageFilter(filter) };
        returnerrifwinerror!(hn, "Could not push storage filter on infoqueue.");
        Ok(())
    }
}
