extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use entity_model;
use game_context::SGameContext;
use render;
use utils::{STransform};

pub fn create(
    game_context: &SGameContext,
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .with_mmm(|entities, render, em| {
            let ent = entities.create_entity()?;

            let model = render.new_model_from_obj("assets/first_test_asset.obj", 1.0, true)?;

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model)?;
            entities.set_location(game_context, ent, starting_location);

            Ok(ent)
        })
}