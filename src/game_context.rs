use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};

use camera;
use databucket::{SDataBucket};
use editmode;
use input;
use niced3d12 as n12;
use safewindows;
use rustywindows;
use render;

#[derive(PartialEq)]
pub enum EMode {
    Play,
    Edit,
}

impl EMode {
    pub fn toggle(&mut self, edit_mode: &mut editmode::EEditMode) {
        match self {
            Self::Play => {
                *self = Self::Edit;
                *edit_mode = editmode::EEditMode::None;
            },
            Self::Edit => {
                *self = Self::Play;
                *edit_mode = editmode::EEditMode::Translation;
            },
        }
    }
}

pub struct SGameContextInt {
    pub cur_frame: u64,

    // -- clock stuff
    pub start_time: u64,
    pub last_frame_start_time: u64,

    pub debug_camera: camera::SCamera,
    pub input: input::SInput,

    pub mode: EMode,
    pub edit_mode_ctxt: Option<editmode::SEditModeContext>, // $$$FRK(TODO): make this non-optional
    pub edit_mode: editmode::EEditMode,

    pub imgui_ctxt: imgui::Context,
    pub show_imgui_demo_window: bool,
}

pub struct SGameContext {
    internal: RefCell<SGameContextInt>,
}

pub struct SFrameContext<'ui> {
    pub total_time_micro_s: u64,
    pub total_time_s: f32,
    pub frame_start_time: u64,

    pub dt_micro_s: u64,
    pub dt_ms: f32,
    pub dt_s: f32,
    pub imgui_ui: imgui::Ui<'ui>,

    pub edit_mode_input: editmode::SEditModeInput,
}

impl SGameContext {
    pub fn new() -> Self {
        let winapi = &rustywindows::winapi;

        let input = input::SInput::new();
        let mut imgui_ctxt = imgui::Context::create();
        input::setup_imgui_key_map(imgui_ctxt.io_mut());

        Self {
            internal: RefCell::new(SGameContextInt{
                cur_frame: 0,

                start_time: winapi.curtimemicroseconds() as u64,
                //frame_start_time: winapi.curtimemicroseconds() as u64,
                last_frame_start_time: winapi.curtimemicroseconds() as u64,

                debug_camera: camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0)),
                input,

                mode: EMode::Edit,
                edit_mode_ctxt: None,
                edit_mode: editmode::EEditMode::None,

                imgui_ctxt,
                show_imgui_demo_window: false,
            }),
        }
    }

    pub fn as_ref(&self) -> &SGameContextInt {
        self.internal.borrow().deref()
    }

    pub fn as_mut(&self) -> &mut SGameContextInt {
        self.internal.borrow_mut().deref_mut()
    }

    pub fn setup_edit_mode_context(&mut self, render: &mut render::SRender) {
        self.edit_mode_ctxt = Some(editmode::SEditModeContext::new(&mut render).unwrap());
    }

    pub fn update_start_frame<'ui>(&'ui mut self, data_bucket: &SDataBucket, window: &n12::SD3D12Window) -> SFrameContext<'ui> {
        let winapi = &rustywindows::winapi;

        // -- handle edit mode toggles
        if self.input.tilde_edge.down() {
            self.mode.toggle(&mut self.edit_mode);
        }

        // -- create per-frame struct
        let frame_start_time = winapi.curtimemicroseconds() as u64;
        let dt_micro_s = frame_start_time - self.last_frame_start_time;
        let total_time_micro_s = frame_start_time - self.start_time;
        let total_time_s = (total_time_micro_s as f32) / 1_000_000.0;
        let dt_ms = (dt_micro_s as f32) / 1_000.0;
        let dt_s = (dt_micro_s as f32) / 1_000_000.0;
        let imgui_ui_for_frame = self.imgui_ctxt.frame();

        // -- $$$FRK(TODO): remove dependency on render
        let edit_mode_input = data_bucket.get::<render::SRender>().with(|render| {
            editmode::SEditModeInput::new_for_frame(&window, &winapi, &self.debug_camera, &render, &self.imgui_ctxt)
        });

        SFrameContext{
            total_time_micro_s,
            total_time_s,
            frame_start_time,

            dt_micro_s,
            dt_ms,
            dt_s,
            imgui_ui: imgui_ui_for_frame,

            edit_mode_input,
        }
    }

    pub fn update_end_frame<'gc>(&'gc mut self, frame_context: SFrameContext<'gc>) {
        self.input.mouse_dx = 0;
        self.input.mouse_dy = 0;

        self.cur_frame += 1;
        self.last_frame_start_time = frame_context.frame_start_time;
    }

    pub fn update_debug_camera(&mut self, frame_context: &SFrameContext) {
        let mut can_rotate_camera = false;
        if let EMode::Play = self.mode {
            can_rotate_camera = true;
        }
        else if self.input.middle_mouse_down {
            can_rotate_camera = true;
        }
        self.debug_camera.update_from_input(&self.input, frame_context.dt_s, can_rotate_camera);
    }

    pub fn update_edit_mode(&mut self, data_bucket: &SDataBucket, frame_context: &SFrameContext) {
        if self.mode == EMode::Edit {
            self.edit_mode = self.edit_mode.update(
                &mut self,
                &frame_context,
                data_bucket,
            );
        }
    }

    pub fn update_io(&mut self, data_bucket: &SDataBucket, frame_context: &SFrameContext, window: &n12::SD3D12Window) -> Result<(), &'static str> {
        let io = self.imgui_ctxt.io_mut(); // for filling out io state
        io.display_size = [window.width() as f32, window.height() as f32];
        io.mouse_pos = [frame_context.edit_mode_input.mouse_window_pos[0] as f32, frame_context.edit_mode_input.mouse_window_pos[1] as f32];

        let mut input_handler = self.input.frame(io);
        loop {
            let msg = window.pollmessage();
            match msg {
                None => break,
                Some(m) => match m {
                    safewindows::EMsgType::Paint => {
                        //println!("Paint!");
                        window.dummyrepaint();
                    }
                    safewindows::EMsgType::KeyDown { key } => {
                        input_handler.handle_key_down_up(key, true);
                    },
                    safewindows::EMsgType::KeyUp { key } => {
                        input_handler.handle_key_down_up(key, false);
                    },
                    safewindows::EMsgType::LButtonDown{ .. } => {
                        input_handler.handle_lmouse_down_up(true);
                    },
                    safewindows::EMsgType::LButtonUp{ .. } => {
                        input_handler.handle_lmouse_down_up(false);
                    },
                    safewindows::EMsgType::MButtonDown{ .. } => {
                        input_handler.handle_mmouse_down_up(true);
                    },
                    safewindows::EMsgType::MButtonUp{ .. } => {
                        input_handler.handle_mmouse_down_up(false);
                    },
                    safewindows::EMsgType::Input{ raw_input } => {
                        if let safewindows::rawinput::ERawInputData::Mouse{data} = raw_input.data {
                            input_handler.handle_mouse_move(data.last_x, data.last_y);
                        }
                    },
                    safewindows::EMsgType::Size => {
                        //println!("Size");
                        let rect: safewindows::SRect = window.raw().getclientrect()?;
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        data_bucket.get_renderer().with_mut(|render: &mut render::SRender| {
                            render.resize_window(&mut window, newwidth, newheight)
                        })?;
                    }
                    safewindows::EMsgType::Invalid => (),
                },
            }
        }

        Ok(())
    }
}