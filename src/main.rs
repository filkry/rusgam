extern crate arrayvec;
extern crate nalgebra_glm as glm;
extern crate tinytga;
extern crate tobj;
extern crate winapi;
extern crate wio;
extern crate bitflags;
extern crate serde_json;
extern crate serde;
extern crate imgui;
extern crate gltf;

//mod math;
#[macro_use]
mod safewindows;
mod allocate;
mod animation;
mod bvh;
mod collections;
mod databucket;
mod directxgraphicssamples;
mod editmode;
mod entity;
mod entity_animation;
mod entity_model;
mod game_context;
mod gjk;
mod input;
mod niced3d12;
mod rustywindows;
mod typeyd3d12;
mod utils;
mod enumflags;
mod camera;
mod model;
mod render;

mod entitytypes;

// -- std includes
//use std::cell::RefCell;
//use std::mem::size_of;
//use std::io::Write;
//use std::rc::Rc;
//use std::ops::{Deref, DerefMut};

// -- crate includes
//use arrayvec::{ArrayVec};
//use serde::{Serialize, Deserialize};
use glm::{Vec4};

use animation::{SAnimationLoader};
use allocate::{STACK_ALLOCATOR, SYSTEM_ALLOCATOR, SMemVec};
use databucket::{SDataBucket};
use entity::{SEntityBucket};
use game_context::{SGameContext, SFrameContext};
use niced3d12 as n12;
use typeyd3d12 as t12;
//use allocate::{SMemVec, STACK_ALLOCATOR};
use utils::{STransform};
//use model::{SModel, SMeshLoader, STextureLoader};

fn update_frame<'gc>(game_context: &'gc mut SGameContext, data_bucket: &SDataBucket, window: &n12::SD3D12Window) -> Result<SFrameContext<'gc>, &'static str> {
    let fc = game_context.update_start_frame(data_bucket, window);
    game_context.update_debug_camera(&fc);
    game_context.update_edit_mode(data_bucket, &fc);

    Ok(fc)
}

fn main_d3d12() -> Result<(), &'static str> {
    render::compile_shaders_if_changed();

    let mut game_context = SGameContext::new();
    let mut render = render::SRender::new(&rustywindows::winapi, &mut game_context.as_ref().imgui_ctxt)?;
    game_context.setup_edit_mode_context(&mut render);

    // -- setup window
    let windowclass_result = rustywindows::winapi.rawwinapi().registerclassex("rusgam");
    if let Err(e) = windowclass_result {
        println!("Failed to make windowclass, error code {:?}", e);
        return Err("failed to make windowclass");
    }
    let windowclass = windowclass_result.unwrap();

    let mut window = render.create_window(&windowclass, "rusgam", 1600, 900)?;

    window.init_render_target_views(render.device())?;
    window.show();

    let mut data_bucket = databucket::SDataBucket::new(256, &SYSTEM_ALLOCATOR);

    data_bucket.add(SEntityBucket::new(67485, 16));
    data_bucket.add(SAnimationLoader::new(&SYSTEM_ALLOCATOR, 64));
    data_bucket.add(render);
    data_bucket.add(entity_model::SBucket::new(&SYSTEM_ALLOCATOR, 1024)?);
    data_bucket.add(entity_animation::SBucket::new(&SYSTEM_ALLOCATOR, 1024)?);
    data_bucket.add(bvh::STree::new());

    let rotating_entity = entitytypes::testtexturedcubeentity::create(
        &game_context,
        &data_bucket, Some("tst_rotating"),
        STransform::new_translation(&glm::Vec3::new(0.0, 0.0, 0.0)))?;
    entitytypes::testtexturedcubeentity::create(
        &game_context,
        &data_bucket, Some("tst_textured_cube"),
        STransform::new_translation(&glm::Vec3::new(3.0, 0.0, 0.0)))?;
    entitytypes::flatshadedcubeentity::create(
        &game_context,
        &data_bucket, Some("tst_coloured_cube"), Some(glm::Vec4::new(1.0, 0.0, 0.0, 0.9)),
        STransform::new_translation(&glm::Vec3::new(0.0, 2.0, 0.0)))?;
    entitytypes::testopenroomentity::create(
        &game_context,
        &data_bucket, Some("tst_room"),
        STransform::new_translation(&glm::Vec3::new(0.0, -2.0, 0.0)))?;
    let skinned_entity = entitytypes::tstskinnedentity::create(
        &game_context,
        &data_bucket, Some("tst_skinned_entity"), Some(glm::Vec4::new(1.0, 1.0, 1.0, 1.0)),
        STransform::new_translation(&glm::Vec3::new(-3.0, 2.0, 0.0)))?;

    data_bucket.get::<entity_animation::SBucket>()
        .and::<animation::SAnimationLoader>()
        .and::<render::SRender>()
        .with_mmc(|ea, anim_loader, render| {
            let handle = ea.handle_for_entity(skinned_entity).unwrap();
            let asset_file_path = "assets/test_armature_animation.gltf";
            ea.play_animation(handle, anim_loader, render.mesh_loader(), asset_file_path, 0.0);
        });

    let mut draw_selected_bvh  = false;

    let mut gjk_debug = gjk::SGJKDebug::new(&data_bucket);

    let should_quit = false;

    // -- update loop
    while !should_quit {
        let frame_context = update_frame(&mut game_context, &data_bucket, &window)?;

        let view_matrix = game_context.as_ref().debug_camera.world_to_view_matrix();

        // update edit mode

        // -- update IMGUI
        if let game_context::EMode::Edit = game_context.as_ref().mode {
            if game_context.as_ref().show_imgui_demo_window {
                let mut opened = true;
                frame_context.imgui_ui.show_demo_window(&mut opened);
            }

            frame_context.imgui_ui.main_menu_bar(|| {
                frame_context.imgui_ui.menu(imgui::im_str!("Misc"), true, || {
                    if imgui::MenuItem::new(imgui::im_str!("Toggle Demo Window")).build(&frame_context.imgui_ui) {
                        game_context.as_mut().show_imgui_demo_window = !game_context.as_ref().show_imgui_demo_window;
                    }
                });

                data_bucket.get_bvh().with(|bvh: &bvh::STree<entity::SEntityHandle>| {
                    bvh.imgui_menu(&frame_context.imgui_ui, &mut draw_selected_bvh);
                });

                gjk_debug.imgui_menu(&frame_context.imgui_ui, &data_bucket, game_context.as_ref().edit_mode_ctxt.unwrap().editing_entity(), Some(rotating_entity));

            });

            if let Some(e) = game_context.as_ref().edit_mode_ctxt.unwrap().editing_entity() {
                data_bucket.get_entities().with_mut(|entities: &mut SEntityBucket| {
                    entities.show_imgui_window(e, &frame_context.imgui_ui);
                });
            }
        }

        // -- draw selected object's BVH heirarchy
        data_bucket.get::<render::SRender>()
            .and::<entity_model::SBucket>()
            .and::<bvh::STree<entity::SEntityHandle>>()
            .with_mcc(|render, em, bvh| {
                if draw_selected_bvh {
                    if let Some(e) = game_context.as_ref().edit_mode_ctxt.unwrap().editing_entity() {
                        STACK_ALLOCATOR.with(|sa| {
                            let model_handle = em.handle_for_entity(e).unwrap();

                            let mut aabbs = SMemVec::new(sa, 32, 0).unwrap();
                            bvh.get_bvh_heirarchy_for_entry(em.get_bvh_entry(model_handle).unwrap(), &mut aabbs);
                            for aabb in aabbs.as_slice() {
                                render.temp().draw_aabb(aabb, &Vec4::new(1.0, 0.0, 0.0, 1.0), true);
                            }
                        });
                    }
                }
            });

        // -- draw selected object colliding/not with rotating_entity
        if let Some(e) = game_context.as_ref().edit_mode_ctxt.unwrap().editing_entity() {
            STACK_ALLOCATOR.with(|sa| {
                data_bucket.get::<render::SRender>()
                    .and::<entity::SEntityBucket>()
                    .and::<entity_model::SBucket>()
                    .with_mcc(|render, entities, em| {
                        let e_model_handle = em.handle_for_entity(e).unwrap();
                        let rot_model_handle = em.handle_for_entity(rotating_entity).unwrap();
                        let loc = entities.get_entity_location(e);

                        let world_verts = {
                            let model = em.get_model(e_model_handle);
                            let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                            let mut world_verts = SMemVec::new(sa, mesh_local_vs.len(), 0).unwrap();

                            for v in mesh_local_vs.as_slice() {
                                world_verts.push(loc.mul_point(&v));
                            }

                            world_verts
                        };

                        let rot_box_world_verts = {
                            let model = em.get_model(rot_model_handle);
                            let loc = entities.get_entity_location(rotating_entity);
                            let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                            let mut world_verts = SMemVec::new(sa, mesh_local_vs.len(), 0).unwrap();

                            for v in mesh_local_vs.as_slice() {
                                world_verts.push(loc.mul_point(&v));
                            }

                            world_verts
                        };

                        if gjk::gjk(world_verts.as_slice(), rot_box_world_verts.as_slice()) {
                            render.temp().draw_sphere(&loc.t, 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.1), true, None);
                        }
                    });
            });
        }

        // -- update bvh
        data_bucket.get::<bvh::STree<entity::SEntityHandle>>()
            .and::<entity_model::SBucket>()
            .and::<SEntityBucket>()
            .and::<render::SRender>()
            .with_mmcc(|bvh, entity_model, entities, render| {
                // -- $$$FRK(TODO): only update dirty
                for i in 0..entity_model.models.len() {
                    let model_handle : entity_model::SHandle = i;

                    let entity_handle = entity_model.get_entity(model_handle);
                    if entities.get_location_update_frame(entity_handle) != game_context.as_ref().cur_frame {
                        continue;
                    }

                    let mesh = entity_model.get_model(model_handle).mesh;
                    let identity_aabb = render.mesh_loader().get_mesh_local_aabb(mesh);

                    let location = entities.get_entity_location(entity_handle);

                    let transformed_aabb = utils::SAABB::transform(&identity_aabb, &location);

                    if let Some(bvh_entry) = entity_model.get_bvh_entry(model_handle) {
                        bvh.update_entry(bvh_entry, &transformed_aabb);
                    }
                    else {
                        let new_bvh_handle = bvh.insert(entity_handle, &transformed_aabb, None);
                        entity_model.set_bvh_entry(model_handle, new_bvh_handle);
                    }
                }
            });

        // -- update animation
        data_bucket.get::<entity_animation::SBucket>()
            .and::<animation::SAnimationLoader>()
            .with_mc(|e_animation, anim_loader| {
                e_animation.update_joints(anim_loader, frame_context.total_time_s);
            });

        // -- draw skeleton of selected entity
        /*
        STACK_ALLOCATOR.with(|sa| {
            data_bucket.get::<render::SRender>().unwrap()
                .and::<entity_model::SBucket>(&data_bucket).unwrap()
                .and::<SEntityBucket>(&data_bucket).unwrap()
                .with_mcc(|render: &mut render::SRender, em: &entity_model::SBucket, entities: &SEntityBucket| {
                    if let Some(e) = editmode_ctxt.editing_entity() {
                        let loc = entities.get_entity_location(e);
                        let model_handle = em.handle_for_entity(e).unwrap();
                        let model = em.get_model(model_handle);

                        let mut joint_locs = SMemVec::new(sa, 128, 0).unwrap();

                        if let Some(bind_joints) = render.mesh_loader().get_mesh_bind_joints(model.mesh) {
                            if let Some(model_skinning) = entities.get_model_skinning(e) {
                                for (ji, joint) in bind_joints.as_ref().iter().enumerate() {
                                    let mut local_to_root = model_skinning.cur_joints_to_parents[ji];
                                    let mut next_idx_opt = joint.parent_idx;
                                    while let Some(next_idx) = next_idx_opt {
                                        local_to_root = STransform::mul_transform(&bind_joints[next_idx].local_to_parent, &local_to_root);
                                        next_idx_opt = bind_joints[next_idx].parent_idx;
                                    }

                                    let local_to_world = STransform::mul_transform(&loc, &local_to_root);
                                    joint_locs.push(local_to_world);
                                }
                            }
                        }

                        for joint_loc in joint_locs.as_ref() {
                            let end = joint_loc.t + glm::quat_rotate_vec3(&joint_loc.r, &Vec3::new(0.0, 1.0, 0.0));
                            render.temp().draw_line(&joint_loc.t, &end, &Vec4::new(0.0, 1.0, 0.0, 1.0), true, None);
                        }
                    }
                });
        });
        */

        // -- render frame
        let imgui_draw_data = frame_context.imgui_ui.render();

        data_bucket.get::<render::SRender>()
            .and::<SEntityBucket>()
            .and::<entity_animation::SBucket>()
            .and::<entity_model::SBucket>()
            .with_mmmc(|render, entities, entity_animation, entity_model| {
                let render_result = render.render_frame(&mut window, &view_matrix, entities, entity_animation, entity_model, Some(&imgui_draw_data));
                match render_result {
                    Ok(_) => {},
                    Err(e) => {
                        println!("ERROR: render failed with error '{}'", e);
                        panic!();
                    },
                }
            });

        // -- $$$FRK(TODO): framerate is uncapped

        game_context.update_io(&data_bucket, &frame_context, &window);

        game_context.update_end_frame(frame_context);

        panic!("must update should_quit");

        // -- increase frame time for testing
        //std::thread::sleep(std::time::Duration::from_millis(111));
    }

    // -- wait for all commands to clear
    data_bucket.get_renderer().with_mut(|render: &mut render::SRender| {
        render.flush()
    })?;

    // -- find out what we leaked
    //drop(render);
    //let debug_interface = t12::SDXGIDebugInterface::new()?;
    //debug_interface.report_live_objects();

    Ok(())
}

fn debug_test() {}

fn main() {
    use std::panic;
    panic::set_hook(Box::new(|_| {
        safewindows::break_if_debugging();
    }));

    debug_test();

    let result = main_d3d12();
    if let Err(e) = result {
        println!("Aborted with error: {:?}", e);
    }
}
