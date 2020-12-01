use allocate::{SYSTEM_ALLOCATOR, SAllocatorRef};
use databucket::{SDataBucket};
use rustywindows;
use niced3d12 as n12;

pub struct SGameContext {
    pub cur_frame: u64,
    pub start_time_micro_s: i64,
    pub last_frame_start_time_micro_s: i64,

    pub data_bucket: SDataBucket,
}

pub struct SFrameContext<'ui> {
    pub start_time_micro_s: i64,
    pub dt_micro_s: i64,
    pub dt_s: f32,
    pub total_time_s: f32,

    pub window_width: u32,
    pub window_height: u32,

    pub imgui_ui: Option<imgui::Ui<'ui>>, // -- goes away partway through the frame
    pub imgui_want_capture_mouse: bool,

    pub data_bucket: SDataBucket,
}

impl SGameContext {
    pub fn new(winapi: &rustywindows::SWinAPI) -> Self {
        Self{
            cur_frame: 0,
            start_time_micro_s: winapi.curtimemicroseconds(),
            last_frame_start_time_micro_s: winapi.curtimemicroseconds(),
            data_bucket: SDataBucket::new(256, &SYSTEM_ALLOCATOR()),
        }
    }

    pub fn start_frame<'ui>(
        &mut self,
        winapi: &rustywindows::SWinAPI,
        window: &n12::SD3D12Window,
        imgui_ctxt: &'ui mut imgui::Context,
        allocator: &SAllocatorRef
    ) -> SFrameContext<'ui> {
        let start_time_micro_s = winapi.curtimemicroseconds();
        let dt_micro_s = start_time_micro_s - self.last_frame_start_time_micro_s;
        let dt_s = (dt_micro_s as f32) / 1_000_000.0;

        let total_time_micro_s = start_time_micro_s - self.start_time_micro_s;
        let total_time_s = (total_time_micro_s as f32) / 1_000_000.0;

        let imgui_want_capture_mouse = imgui_ctxt.io().want_capture_mouse;

        SFrameContext {
            start_time_micro_s,
            dt_micro_s,
            dt_s,
            total_time_s,

            window_width: window.width(),
            window_height: window.height(),

            imgui_ui: Some(imgui_ctxt.frame()),
            imgui_want_capture_mouse,

            data_bucket: SDataBucket::new(32, allocator),
        }
    }

    pub fn end_frame(&mut self, frame_context: SFrameContext) {
        self.last_frame_start_time_micro_s = frame_context.start_time_micro_s;
    }
}
