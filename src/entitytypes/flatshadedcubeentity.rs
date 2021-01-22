use serde::{Serialize, Deserialize};

use crate::entity::*;
use crate::entity_model;
use crate::entitytypes::{EEntityType};
use crate::game_context::{SGameContext};
use crate::math::{Vec4};
use crate::render;
use crate::utils::{STransform};

#[derive(Serialize, Deserialize)]
pub struct SInit {
    debug_name: Option<String>,
    diffuse_colour: Option<Vec4>,
    starting_location: STransform,
}

pub fn create_from_init(gc: &SGameContext, init: &SInit) -> Result<SEntityHandle, &'static str> {
    create(gc, init.debug_name.as_deref(), init.diffuse_colour, init.starting_location)
}

pub fn create(
    gc: &SGameContext,
    debug_name: Option<&str>,
    diffuse_colour: Option<Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    gc.data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .with_mmm(|entities, render, em| {
            let ent = entities.create_entity(EEntityType::FlatShadedCube)?;

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

impl SInit {
    pub fn new_from_entity(gc: &SGameContext, entity: SEntityHandle) -> Self {
        gc.data_bucket.get::<SEntityBucket>()
            .and::<entity_model::SBucket>()
            .with_cc(|entities, em| {
                assert_eq!(entities.get_entity_type(entity), EEntityType::FlatShadedCube);

                let debug_name = entities.get_entity_debug_name(entity).map(|n| {
                    let name_raw_str = unsafe{ n._debug_ptr.as_ref().unwrap() };
                    String::from(name_raw_str)
                });
                let m_handle = em.handle_for_entity(entity).expect("somehow model wasn't created");
                let diffuse_colour = Some(em.get_model(m_handle).diffuse_colour);
                let starting_location = entities.get_entity_location(entity);

                Self{
                    debug_name,
                    diffuse_colour,
                    starting_location,
                }
            })
    }
}