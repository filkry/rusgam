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

    data_bucket.get::<SEntityBucket>().unwrap()
        .and::<render::SRender>(data_bucket).unwrap()
        .and::<entity_model::SBucket>(data_bucket).unwrap()
        .and::<SGameContext>(data_bucket).unwrap()
        .with_mmmc(|entities: &mut SEntityBucket, render: &mut render::SRender, em: &mut entity_model::SBucket, gc: &SGameContext| {
            let ent = entities.create_entity()?;

            let model = render.new_model_from_obj("assets/test_open_room.obj", 1.0, true)?;

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            em.add_instance(ent, model)?;
            entities.set_location(gc, ent, starting_location);

            Ok(ent)
        })
}