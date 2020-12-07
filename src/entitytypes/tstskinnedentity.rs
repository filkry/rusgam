use serde::{Serialize, Deserialize};

use animation;
use entity::*;
use entity_model;
use entity_animation;
use entitytypes::{EEntityType};
use game_context::{SGameContext};
use math::{Vec4};
use render;
use utils::{STransform};

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
    game_context: &SGameContext,
    debug_name: Option<&str>,
    diffuse_colour: Option<Vec4>,
    starting_location: STransform,
) -> Result<SEntityHandle, &'static str> {

    game_context.data_bucket.get::<SEntityBucket>()
        .and::<render::SRender>()
        .and::<entity_model::SBucket>()
        .and::<entity_animation::SBucket>()
        .and::<animation::SAnimationLoader>()
        .with_mmmmm(|entities, render, e_model, e_animation, anim_loader| {
            let ent = entities.create_entity(EEntityType::TestSkinnedEntity)?;

            let mut model = render.new_model_from_gltf("assets/test_armature.gltf", 1.0, true)?;
            if let Some(c) = diffuse_colour {
                model.diffuse_colour = c;
            }

            if let Some(n) = debug_name {
                entities.set_entity_debug_name(ent, n);
            }

            let model_handle = e_model.add_instance(ent, model)?;
            let anim_handle = e_animation.add_instance(ent, (&e_model, model_handle), render.mesh_loader())?;

            entities.set_location(game_context, ent, starting_location);

            {
                let asset_file_path = "assets/test_armature_animation.gltf";
                e_animation.play_animation(anim_handle, anim_loader, render.mesh_loader(), asset_file_path, 0.0);
            }

            Ok(ent)
        })
}

impl SInit {
    pub fn new_from_entity(gc: &SGameContext, entity: SEntityHandle) -> Self {
        gc.data_bucket.get::<SEntityBucket>()
            .and::<entity_model::SBucket>()
            .with_cc(|entities, em| {
                assert_eq!(entities.get_entity_type(entity), EEntityType::TestSkinnedEntity);

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
