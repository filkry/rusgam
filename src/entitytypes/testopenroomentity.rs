extern crate nalgebra_glm as glm;

use databucket::{SDataBucket};
use entity::*;
use render;
use utils::{STransform};

pub fn create(
    data_bucket: &SDataBucket,
    debug_name: Option<&'static str>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    data_bucket.get_entities().unwrap().with_mut(|entities: &mut SEntityBucket| {
        let ent = entities.create_entity()?;

        let model = data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
            render.new_model("assets/test_open_room.obj", 1.0, true)
        })?;

        if let Some(n) = debug_name {
            entities.set_entity_debug_name(ent, n);
        }

        entities.set_entity_model(ent, model, data_bucket);
        entities.set_entity_location(ent, starting_location, data_bucket);

        Ok(ent)
    })
}