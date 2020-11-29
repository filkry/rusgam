use allocate::{SYSTEM_ALLOCATOR, TMemAllocator};
use databucket::{SDataBucket};
use rustywindows;

pub struct SGameContext<'a> {
    pub cur_frame: u64,
    pub start_time_micro_s: i64,
    pub last_frame_start_time_micro_s: i64,

    pub data_bucket: SDataBucket<'a>,
}

pub struct SFrameContext<'a> {
    pub start_time_micro_s: i64,
    pub dt_micro_s: i64,
    pub dt_s: f32,
    pub total_time_s: f32,

    pub data_bucket: SDataBucket<'a>,
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

    pub fn start_frame<'b>(&mut self, winapi: &rustywindows::SWinAPI, allocator: &'b dyn TMemAllocator) -> SFrameContext<'b> {
        let start_time_micro_s = winapi.curtimemicroseconds();
        let dt_micro_s = start_time_micro_s - self.last_frame_start_time_micro_s;
        let dt_s = (dt_micro_s as f32) / 1_000_000.0;

        let total_time_micro_s = start_time_micro_s - self.start_time_micro_s;
        let total_time_s = (total_time_micro_s as f32) / 1_000_000.0;

        SFrameContext {
            start_time_micro_s,
            dt_micro_s,
            dt_s,
            total_time_s,

            data_bucket: SDataBucket::new(32, allocator),
        }
    }

    pub fn end_frame(&mut self, frame_context: SFrameContext) {
        self.last_frame_start_time_micro_s = frame_context.start_time_micro_s;
    }
}
