extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use entity_model;
use render;
use utils::{STransform};

pub fn create(
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    diffuse_colour: Option<glm::Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get::<SEntityBucket>().unwrap()
        .and::<render::SRender>(data_bucket).unwrap()
        .and::<entity_model::SBucket>(data_bucket).unwrap()
        .with_mmm(|entities: &mut SEntityBucket, render: &mut render::SRender, em: &mut entity_model::SBucket| {
            let ent = entities.create_entity()?;

            let mut model = render.new_model_from_gltf("assets/test_untextured_flat_colour_cube.gltf", 1.0, true)?;
            if let Some(c) = diffuse_colour {
                model.diffuse_colour = c;
            }

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model);
            entities.set_location(ent, starting_location);

            Ok(ent)
        })
}