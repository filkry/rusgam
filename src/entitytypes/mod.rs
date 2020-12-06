use serde::{Serialize, Deserialize};

use entity::{SEntityHandle};
use game_context::{SGameContext};

pub mod flatshadedcubeentity;
pub mod testtexturedcubeentity;
pub mod testopenroomentity;
pub mod tstskinnedentity;

#[derive(Serialize, Deserialize)]
pub enum EEntityInit {
    FlatShadedCube(flatshadedcubeentity::SInit),
}

impl EEntityInit {
    pub fn init(&self, game_context: &SGameContext) -> Result<SEntityHandle, &'static str> {
        match self {
            Self::FlatShadedCube(init) => flatshadedcubeentity::create_from_init(game_context, init),
        }
    }
}