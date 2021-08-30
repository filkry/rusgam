use super::*;

pub struct SFence {
    raw: t12::SFence,

    pub(super) fenceevent: safewindows::SEventHandle,
    pub(super) nextfencevalue: u64,
}

impl SFence {
    pub fn new_from_raw(raw: t12::SFence, evt: safewindows::SEventHandle) -> Self {
        let completedvalue = raw.getcompletedvalue();
        assert!(completedvalue < 1);
        Self {
            raw: raw,
            fenceevent: evt,
            nextfencevalue: 1,
        }
    }

    pub unsafe fn raw(&self) -> &t12::SFence {
        &self.raw
    }

    pub fn last_signalled_value(&self) -> u64 {
        self.nextfencevalue - 1
    }

    pub fn completed_value(&self) -> u64 {
        self.raw.getcompletedvalue()
    }

    pub fn wait_for_value(&self, val: u64) {
        self.wait_for_value_duration(val, <u32>::max_value())
            .unwrap();
    }

    pub fn wait_for_value_duration(&self, val: u64, duration: u32) -> Result<(), &'static str> {
        if self.raw.getcompletedvalue() < val {
            self.raw.seteventoncompletion(val, &self.fenceevent)?;
            self.fenceevent.waitforsingleobject(duration);
        }

        Ok(())
    }
}
