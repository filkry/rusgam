use entity::*;
use entity_model;
use game_context::{SGameContext};
use math::{Vec4};
use render;
use utils::{STransform};

pub fn create(
    gc: &SGameContext,
    debug_name: Option<&'static str>,
    diffuse_colour: Option<Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    gc.data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .with_mmm(|entities, render, em| {
            let ent = entities.create_entity()?;

            let mut model = render.new_model_from_gltf("assets/test_untextured_flat_colour_cube.gltf", 1.0, true)?;
            if let Some(c) = diffuse_colour {
                model.diffuse_colour = c;
            }

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model)?;
            entities.set_location(gc, ent, starting_location);

            Ok(ent)
        })
}