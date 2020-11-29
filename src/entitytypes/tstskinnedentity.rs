extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use entity_model;
use entity_animation;
use game_context::{SGameContext};
use render;
use utils::{STransform};

pub fn create(
    game_context: &SGameContext,
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    diffuse_colour: Option<glm::Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .and::<entity_animation::SBucket>()
        .with_mmmm(|entities, render, e_model, e_animation| {
            let ent = entities.create_entity()?;

            let mut model = render.new_model_from_gltf("assets/test_armature.gltf", 1.0, true)?;
            if let Some(c) = diffuse_colour {
                model.diffuse_colour = c;
            }

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            let model_handle = e_model.add_instance(ent, model)?;
            e_animation.add_instance(ent, (&e_model, model_handle), render.mesh_loader())?;

            entities.set_location(game_context, ent, starting_location);

            Ok(ent)
        })
}