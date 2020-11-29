use editmode::{EEditMode, SEditModeContext};
use render;

#[derive(PartialEq)]
pub enum EMode {
    Play,
    Edit,
}

impl EMode {
    pub fn toggle(&mut self, edit_mode: &mut EEditMode) {
        match self {
            Self::Play => {
                *self = Self::Edit;
                *edit_mode = EEditMode::None;
            },
            Self::Edit => {
                *self = Self::Play;
                *edit_mode = EEditMode::Translation;
            },
        }
    }
}

pub struct SGameMode {
    pub mode: EMode,
    pub edit_mode: EEditMode,
    pub edit_mode_ctxt: SEditModeContext,

    pub draw_selected_bvh: bool,
    pub show_imgui_demo_window: bool,
}

impl SGameMode {
    pub fn new(render: &mut render::SRender) -> Self {
        Self{
            mode: EMode::Edit,
            edit_mode: EEditMode::None,
            edit_mode_ctxt: SEditModeContext::new(render).unwrap(),
            draw_selected_bvh: false,
            show_imgui_demo_window: false,
        }
    }
}