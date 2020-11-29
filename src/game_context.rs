use allocate::{SYSTEM_ALLOCATOR};
use databucket::{SDataBucket};
use rustywindows;

pub struct SGameContext<'a> {
    pub cur_frame: u64,
    pub start_time_micro_s: i64,
    pub last_frame_start_time_micro_s: i64,

    pub data_bucket: SDataBucket<'a>,
}

pub struct SFrameContext {
    pub start_time_micro_s: i64,
}

impl<'a> SGameContext<'a> {
    pub fn new(winapi: &rustywindows::SWinAPI) -> Self {
        Self{
            cur_frame: 0,
            start_time_micro_s: winapi.curtimemicroseconds(),
            last_frame_start_time_micro_s: winapi.curtimemicroseconds(),
            data_bucket: SDataBucket::new(256, &SYSTEM_ALLOCATOR),
        }
    }

    pub fn start_frame(&mut self, winapi: &rustywindows::SWinAPI) -> SFrameContext {
        SFrameContext {
            start_time_micro_s: winapi.curtimemicroseconds(),
        }
    }

    pub fn end_frame(&mut self, frame_context: SFrameContext) {
        self.last_frame_start_time_micro_s = frame_context.start_time_micro_s;
    }
}
