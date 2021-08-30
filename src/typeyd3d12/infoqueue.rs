use super::*;

pub struct SInfoQueue {
    infoqueue: win::ID3D12InfoQueue,
}

impl SInfoQueue {
    pub unsafe fn new_from_raw(raw: win::ID3D12InfoQueue) -> Self {
        Self { infoqueue: raw }
    }

    pub fn set_break_on_severity(&self, id: win::D3D12_MESSAGE_SEVERITY, val: bool) {
        unsafe {
            self.infoqueue.SetBreakOnSeverity(id, val);
        }
    }

    pub fn pushstoragefilter(
        &self,
        filter: &mut win::D3D12_INFO_QUEUE_FILTER,
    ) -> Result<(), &'static str> {
        let hn = unsafe { self.infoqueue.PushStorageFilter(filter) };
        returnerrifwinerror!(hn, "Could not push storage filter on infoqueue.");
        Ok(())
    }
}
