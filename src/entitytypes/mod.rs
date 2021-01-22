use serde::{Serialize, Deserialize};

use crate::entity::{SEntityBucket, SEntityHandle};
use crate::game_context::{SGameContext};

pub mod flatshadedcubeentity;
pub mod testtexturedcubeentity;
pub mod testopenroomentity;
pub mod tstskinnedentity;

#[derive(Serialize, Deserialize)]
pub enum EEntityInit {
    FlatShadedCube(flatshadedcubeentity::SInit),
    TestOpenRoom(testopenroomentity::SInit),
    TestTexturedCube(testtexturedcubeentity::SInit),
    TestSkinnedEntity(tstskinnedentity::SInit),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EEntityType {
    Invalid,
    FlatShadedCube,
    TestOpenRoom,
    TestTexturedCube,
    TestSkinnedEntity,
}

/*
trait TEntityType {
    type TInit;

    pub fn init_from_entity(gc: &SGameContext, entity: SEntityHandle) -> Self::TInit;
    pub fn entity_from_init(gc: &SGameContext, init: &Self::TInit) -> Result<SEntityHandle, &'static str>;
}
*/

impl EEntityInit {
    pub fn new_from_entity(game_context: &SGameContext, entity: SEntityHandle) -> EEntityInit {
        let entity_type = game_context.data_bucket.get::<SEntityBucket>()
            .with(|entities| {
                entities.get_entity_type(entity)
            });
        match entity_type {
            EEntityType::FlatShadedCube => EEntityInit::FlatShadedCube(flatshadedcubeentity::SInit::new_from_entity(game_context, entity)),
            EEntityType::TestOpenRoom => EEntityInit::TestOpenRoom(testopenroomentity::SInit::new_from_entity(game_context, entity)),
            EEntityType::TestTexturedCube => EEntityInit::TestTexturedCube(testtexturedcubeentity::SInit::new_from_entity(game_context, entity)),
            EEntityType::TestSkinnedEntity => EEntityInit::TestSkinnedEntity(tstskinnedentity::SInit::new_from_entity(game_context, entity)),
            EEntityType::Invalid => panic!("Trying to create init for invalid entity"),
        }
    }

    pub fn init(&self, game_context: &SGameContext) -> Result<SEntityHandle, &'static str> {
        match self {
            Self::FlatShadedCube(init) => flatshadedcubeentity::create_from_init(game_context, init),
            Self::TestOpenRoom(init) => testopenroomentity::create_from_init(game_context, init),
            Self::TestTexturedCube(init) => testtexturedcubeentity::create_from_init(game_context, init),
            Self::TestSkinnedEntity(init) => tstskinnedentity::create_from_init(game_context, init),
        }
    }
}