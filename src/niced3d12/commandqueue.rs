use super::*;

use arrayvec::ArrayVec;

pub struct SCommandQueue {
    raw: t12::SCommandQueue,

    fence: SFence,

    commandlisttype: t12::ECommandListType,

    pub(super) debug_name: ArrayVec<[u16; 64]>,
}

impl SCommandQueue {
    pub fn new_from_raw(
        raw: t12::SCommandQueue,
        fence: SFence,
        type_: t12::ECommandListType,
    ) -> Self {
        Self {
            raw: raw,
            fence: fence,
            commandlisttype: type_,
            debug_name: ArrayVec::new(),
        }
    }

    pub unsafe fn raw(&self) -> &t12::SCommandQueue {
        &self.raw
    }

    pub unsafe fn set_debug_name(&mut self, str_: &'static str) {
        self.raw().raw().SetName(str_).expect("who knows why this would fail");
    }

    pub fn type_(&self) -> t12::ECommandListType {
        self.commandlisttype
    }

    pub fn execute_command_lists(
        &self, // -- verified thread safe in docs
        lists: &mut [&mut SCommandList],
    ) -> Result<(), &'static str> {
        unsafe {
            let mut raw_lists = ArrayVec::<[&mut t12::SCommandList; 12]>::new();
            for list in lists {
                list.raw().close()?;
                raw_lists.push(list.raw_mut());
            }
            self.raw.execute_command_lists(raw_lists.as_ref())
        };
        Ok(())
    }

    pub fn signal(
        &self, // -- I'm assuming this is safe
        fence: &mut SFence,
    ) -> Result<u64, &'static str> {
        let result = fence.nextfencevalue;
        self.raw
            .signal(unsafe { fence.raw() }, fence.nextfencevalue)?;
        fence.nextfencevalue += 1;
        Ok(result)
    }

    pub fn internal_fence_value(&self) -> u64 {
        unsafe { self.fence.raw().getcompletedvalue() }
    }

    pub fn signal_internal_fence(&mut self) -> Result<u64, &'static str> {
        let result = self.fence.nextfencevalue;
        self.raw
            .signal(unsafe { self.fence.raw() }, self.fence.nextfencevalue)?;
        self.fence.nextfencevalue += 1;
        Ok(result)
    }

    pub fn wait_for_internal_fence_value(&self, value: u64) {
        self.fence.wait_for_value(value);
    }

    pub fn gpu_wait(&self, fence: &SFence, value: u64) -> Result<(), &'static str> {
        self.raw.wait(unsafe { fence.raw() }, value)?;
        Ok(())
    }

    pub fn flush_blocking(&mut self) -> Result<(), &'static str> {
        let lastfencevalue = self.signal_internal_fence()?;
        self.fence.wait_for_value(lastfencevalue);
        Ok(())
    }
}

impl Drop for SCommandQueue {
    fn drop(&mut self) {
        self.flush_blocking().unwrap();
    }
}
