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
        for ch in str_.encode_utf16() {
            self.debug_name.push(ch);
        }
        self.debug_name.push('\0' as u16);

        self.raw().raw().SetName(&self.debug_name[0]);
    }

    pub fn type_(&self) -> t12::ECommandListType {
        self.commandlisttype
    }

    pub fn execute_command_list(
        &self, // -- verified thread safe in docs
        list: &mut SCommandList,
    ) -> Result<(), &'static str> {
        unsafe {
            list.raw().close()?;
            self.raw.executecommandlist(&list.raw())
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
