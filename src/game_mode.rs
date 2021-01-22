use crate::editmode::{EEditMode, SEditModeContext};
use crate::game_context::{SGameContext};
use crate::input;
use crate::render;

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

pub fn update_toggle_mode(game_context: &SGameContext) {
    game_context.data_bucket.get::<SGameMode>()
        .and::<input::SInput>()
        .with_mc(|game_mode, input| {
            if input.tilde_edge.down() {
                game_mode.mode.toggle(&mut game_mode.edit_mode);
            }
        });
}