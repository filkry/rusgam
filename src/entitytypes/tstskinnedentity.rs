extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use entity_model;
use render;
use utils::{STransform, SGameContext};

pub fn create(
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    diffuse_colour: Option<glm::Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get::<SEntityBucket>().unwrap()
        .and::<render::SRender>(data_bucket).unwrap()
        .and::<entity_model::SBucket>(data_bucket).unwrap()
        .and::<SGameContext>(data_bucket).unwrap()
        .with_mmmc(|
            entities: &mut SEntityBucket,
            render: &mut render::SRender,
            em: &mut entity_model::SBucket,
            gc: &SGameContext,
        | {
            let ent = entities.create_entity()?;

            let (mut model, model_skinning) = {
                let model = render.new_model_from_gltf("assets/test_armature.gltf", 1.0, true).unwrap();
                let model_skinning = render.bind_model_skinning(&model).unwrap();

                (model, model_skinning)
            };
            if let Some(c) = diffuse_colour {
                model.diffuse_colour = c;
            }

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model)?;
            entities.set_entity_model_skinning(ent, model_skinning);
            entities.set_location(gc, ent, starting_location);

            Ok(ent)
        })
}