use editmode::{EEditMode};

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
}

impl SGameMode {
    pub fn new() -> Self {
        Self{
            mode: EMode::Edit,
            edit_mode: EEditMode::None,
        }
    }
}