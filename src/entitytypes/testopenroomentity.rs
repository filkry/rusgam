use serde::{Serialize, Deserialize};

use entity::*;
use entity_model;
use entitytypes::{EEntityType};
use game_context::{SGameContext};
use render;
use utils::{STransform};

#[derive(Serialize, Deserialize)]
pub struct SInit {
    debug_name: Option<String>,
    starting_location: STransform,
}

pub fn create_from_init(gc: &SGameContext, init: &SInit) -> Result<SEntityHandle, &'static str> {
    create(gc, init.debug_name.as_deref(), init.starting_location)
}

pub fn create(
    game_context: &SGameContext,
    debug_name: Option<&str>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    game_context.data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .with_mmm(|entities, render, em| {
            let ent = entities.create_entity(EEntityType::TestOpenRoom)?;

            let model = render.new_model_from_obj("assets/test_open_room.obj", 1.0, true)?;

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model)?;
            entities.set_location(game_context, ent, starting_location);

            Ok(ent)
        })
}

impl SInit {
    pub fn new_from_entity(gc: &SGameContext, entity: SEntityHandle) -> Self {
        gc.data_bucket.get::<SEntityBucket>()
            .with(|entities| {
                assert_eq!(entities.get_entity_type(entity), EEntityType::TestOpenRoom);

                let debug_name = entities.get_entity_debug_name(entity).map(|n| {
                    let name_raw_str = unsafe{ n._debug_ptr.as_ref().unwrap() };
                    String::from(name_raw_str)
                });
                let starting_location = entities.get_entity_location(entity);

                Self{
                    debug_name,
                    starting_location,
                }
            })
    }
}
