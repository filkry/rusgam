extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use render;
use utils::{STransform};

pub fn create(
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    diffuse_colour: Option<glm::Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get_entities().unwrap().with_mut(|entities: &mut SEntityBucket| {
        let ent = entities.create_entity()?;

        let mut model = data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
            render.new_model_from_gltf("assets/test_armature.gltf", 1.0, true)
        })?;
        if let Some(c) = diffuse_colour {
            model.diffuse_colour = c;
        }

        if let Some(n) = debug_name {
            entities.set_entity_debug_name(ent, n);
        }

        entities.set_entity_model(ent, model, data_bucket);
        entities.set_entity_location(ent, starting_location, data_bucket);

        Ok(ent)
    })
}