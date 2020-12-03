use allocate::{SYSTEM_ALLOCATOR, SAllocatorRef};
use databucket::{SDataBucket};
use rustywindows;
use niced3d12 as n12;

pub struct SGameContext {
    pub window: n12::SD3D12Window,

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
    pub imgui_draw_data: Option<&'ui imgui::DrawData>, // -- created partway through the frame
    pub imgui_want_capture_mouse: bool,

    pub data_bucket: SDataBucket,
}

impl SGameContext {
    pub fn new(winapi: &rustywindows::SWinAPI, window: n12::SD3D12Window) -> Self {
        Self{
            window,
            cur_frame: 0,
            start_time_micro_s: winapi.curtimemicroseconds(),
            last_frame_start_time_micro_s: winapi.curtimemicroseconds(),
            data_bucket: SDataBucket::new(256, &SYSTEM_ALLOCATOR()),
        }
    }

    pub fn start_frame<'ui>(
        &mut self,
        winapi: &rustywindows::SWinAPI,
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

            window_width: self.window.width(),
            window_height: self.window.height(),

            imgui_ui: Some(imgui_ctxt.frame()),
            imgui_draw_data: None,
            imgui_want_capture_mouse,

            data_bucket: SDataBucket::new(32, allocator),
        }
    }

    pub fn end_frame(&mut self, frame_context: SFrameContext) {
        self.last_frame_start_time_micro_s = frame_context.start_time_micro_s;
    }
}

impl<'ui> SFrameContext<'ui> {
    pub fn finalize_ui(&mut self) {
        let imgui_draw_data = self.imgui_ui.take().expect("this is where we take it").render();
        self.imgui_draw_data = Some(imgui_draw_data);
    }
}
