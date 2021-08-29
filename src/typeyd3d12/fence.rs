use super::*;

#[derive(Clone)]
pub struct SFence {
    fence: win::ID3D12Fence,
}

impl SFence {
    pub unsafe fn new_from_raw(raw: win::ID3D12Fence) -> Self {
        Self { fence: raw }
    }

    pub unsafe fn raw(&self) -> &win::ID3D12Fence {
        &self.fence
    }

    pub fn getcompletedvalue(&self) -> u64 {
        unsafe { self.fence.GetCompletedValue() }
    }

    pub fn seteventoncompletion(
        &self,
        val: u64,
        event: &safewindows::SEventHandle,
    ) -> Result<(), &'static str> {
        let hn = unsafe { self.fence.SetEventOnCompletion(val, event.raw()) };
        returnerrifwinerror!(hn, "Could not set fence event on completion");
        Ok(())
    }
}
