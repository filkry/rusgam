extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use entity_model;
use render;
use utils::{STransform, SGameContext};

pub fn create(
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .and::<SGameContext>()
        .with_mmmc(|entities, render, em, gc| {
            let ent = entities.create_entity()?;

            let model = render.new_model_from_obj("assets/first_test_asset.obj", 1.0, true)?;

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model)?;
            entities.set_location(gc, ent, starting_location);

            Ok(ent)
        })
}