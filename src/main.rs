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
mod debug_ui;
mod directxgraphicssamples;
mod editmode;
mod entity;
mod entity_animation;
mod entity_model;
mod game_context;
mod game_mode;
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
use allocate::{STACK_ALLOCATOR, SYSTEM_ALLOCATOR, SMemVec, SAllocator};
use entity::{SEntityBucket};
use game_context::{SGameContext, SFrameContext};
use niced3d12 as n12;
use typeyd3d12 as t12;
//use allocate::{SMemVec, STACK_ALLOCATOR};
use utils::{STransform};
//use model::{SModel, SMeshLoader, STextureLoader};


fn update_frame(game_context: &SGameContext, frame_context: &mut SFrameContext) -> Result<(), &'static str> {
    game_mode::update_toggle_mode(game_context);
    camera::update_debug_camera(game_context, frame_context);
    let edit_mode_input = editmode::update_create_input_for_frame(game_context, frame_context);
    frame_context.data_bucket.add(edit_mode_input);
    editmode::update_edit_mode(game_context, frame_context);
    debug_ui::update_debug_main_menu(game_context, frame_context);
    debug_ui::update_debug_entity_menu(game_context, frame_context);

    Ok(())
}

fn main_d3d12() -> Result<(), &'static str> {
    render::compile_shaders_if_changed();

    let winapi = rustywindows::SWinAPI::create();

    let mut imgui_ctxt = imgui::Context::create();

    input::setup_imgui_key_map(imgui_ctxt.io_mut());

    let mut render = render::SRender::new(&winapi, &mut imgui_ctxt)?;

    // -- setup window
    let windowclass_result = winapi.rawwinapi().registerclassex("rusgam");
    if let Err(e) = windowclass_result {
        println!("Failed to make windowclass, error code {:?}", e);
        return Err("failed to make windowclass");
    }
    let windowclass = windowclass_result.unwrap();

    let mut window = render.create_window(&windowclass, "rusgam", 1600, 900)?;
    imgui_ctxt.io_mut().display_size = [window.width() as f32, window.height() as f32];

    window.init_render_target_views(render.device())?;
    window.show();

    let mut game_context = SGameContext::new(&winapi);

    game_context.data_bucket.add(SEntityBucket::new(67485, 16));
    game_context.data_bucket.add(SAnimationLoader::new(SYSTEM_ALLOCATOR(), 64));
    game_context.data_bucket.add(game_mode::SGameMode::new(&mut render));
    game_context.data_bucket.add(render);
    game_context.data_bucket.add(entity_model::SBucket::new(&SYSTEM_ALLOCATOR(), 1024)?);
    game_context.data_bucket.add(entity_animation::SBucket::new(&SYSTEM_ALLOCATOR(), 1024)?);
    game_context.data_bucket.add(bvh::STree::new());
    game_context.data_bucket.add(camera::SDebugFPCamera::new(glm::Vec3::new(0.0, 0.0, -10.0)));
    game_context.data_bucket.add(input::SInput::new());
    game_context.data_bucket.add(gjk::SGJKDebug::new(&game_context.data_bucket));

    let rotating_entity = entitytypes::testtexturedcubeentity::create(
        &game_context, Some("tst_rotating"),
        STransform::new_translation(&glm::Vec3::new(0.0, 0.0, 0.0)))?;
    entitytypes::testtexturedcubeentity::create(
        &game_context, Some("tst_textured_cube"),
        STransform::new_translation(&glm::Vec3::new(3.0, 0.0, 0.0)))?;
    entitytypes::flatshadedcubeentity::create(
        &game_context, Some("tst_coloured_cube"), Some(glm::Vec4::new(1.0, 0.0, 0.0, 0.9)),
        STransform::new_translation(&glm::Vec3::new(0.0, 2.0, 0.0)))?;
    entitytypes::testopenroomentity::create(
        &game_context, Some("tst_room"),
        STransform::new_translation(&glm::Vec3::new(0.0, -2.0, 0.0)))?;
    let skinned_entity = entitytypes::tstskinnedentity::create(
        &game_context, Some("tst_skinned_entity"), Some(glm::Vec4::new(1.0, 1.0, 1.0, 1.0)),
        STransform::new_translation(&glm::Vec3::new(-3.0, 2.0, 0.0)))?;

    game_context.data_bucket.get::<entity_animation::SBucket>()
        .and::<animation::SAnimationLoader>()
        .and::<render::SRender>()
        .with_mmc(|ea, anim_loader, render| {
            let handle = ea.handle_for_entity(skinned_entity).unwrap();
            let asset_file_path = "assets/test_armature_animation.gltf";
            ea.play_animation(handle, anim_loader, render.mesh_loader(), asset_file_path, 0.0);
        });

    let frame_linear_allocator_helper = SAllocator::new(
        allocate::SLinearAllocator::new(SYSTEM_ALLOCATOR(), 128 * 1024 * 1024, 8)?,
    );

    // -- update loop
    while !game_context.data_bucket.get::<input::SInput>().with(|input| input.q_down) {

        let frame_linear_allocator = SAllocator::new(
            allocate::SLinearAllocator::new(frame_linear_allocator_helper.as_ref(), 120 * 1024 * 1024, 8)?,
        );

        let mut frame_context = game_context.start_frame(&winapi, &window, &mut imgui_ctxt, &frame_linear_allocator.as_ref());

        update_frame(&game_context, &mut frame_context)?;

        // -- draw selected object's BVH heirarchy
        STACK_ALLOCATOR.with(|sa| {
            game_context.data_bucket.get::<render::SRender>()
                .and::<entity_model::SBucket>()
                .and::<bvh::STree<entity::SEntityHandle>>()
                .and::<game_mode::SGameMode>()
                .with_mccc(|render, em, bvh, game_mode| {
                    if game_mode.draw_selected_bvh {
                        if let Some(e) = game_mode.edit_mode_ctxt.editing_entity() {
                            let model_handle = em.handle_for_entity(e).unwrap();

                            let mut aabbs = SMemVec::new(&sa.as_ref(), 32, 0).unwrap();
                            bvh.get_bvh_heirarchy_for_entry(em.get_bvh_entry(model_handle).unwrap(), &mut aabbs);
                            for aabb in aabbs.as_slice() {
                                render.temp().draw_aabb(aabb, &Vec4::new(1.0, 0.0, 0.0, 1.0), true);
                            }
                        }
                    }
                });
            });

        // -- draw selected object colliding/not with rotating_entity
        STACK_ALLOCATOR.with(|sa| {
            game_context.data_bucket.get::<render::SRender>()
                .and::<entity::SEntityBucket>()
                .and::<entity_model::SBucket>()
                .and::<game_mode::SGameMode>()
                .with_mccc(|render, entities, em, game_mode| {
                    if let Some(e) = game_mode.edit_mode_ctxt.editing_entity() {
                        let e_model_handle = em.handle_for_entity(e).unwrap();
                        let rot_model_handle = em.handle_for_entity(rotating_entity).unwrap();
                        let loc = entities.get_entity_location(e);

                        let world_verts = {
                            let model = em.get_model(e_model_handle);
                            let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                            let mut world_verts = SMemVec::new(&sa.as_ref(), mesh_local_vs.len(), 0).unwrap();

                            for v in mesh_local_vs.as_slice() {
                                world_verts.push(loc.mul_point(&v));
                            }

                            world_verts
                        };

                        let rot_box_world_verts = {
                            let model = em.get_model(rot_model_handle);
                            let loc = entities.get_entity_location(rotating_entity);
                            let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                            let mut world_verts = SMemVec::new(&sa.as_ref(), mesh_local_vs.len(), 0).unwrap();

                            for v in mesh_local_vs.as_slice() {
                                world_verts.push(loc.mul_point(&v));
                            }

                            world_verts
                        };

                        if gjk::gjk(world_verts.as_slice(), rot_box_world_verts.as_slice()) {
                            render.temp().draw_sphere(&loc.t, 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.1), true, None);
                        }
                    }
                });
        });

        // -- update bvh
        game_context.data_bucket.get::<bvh::STree<entity::SEntityHandle>>()
            .and::<entity_model::SBucket>()
            .and::<SEntityBucket>()
            .and::<render::SRender>()
            .with_mmcc(|bvh, entity_model, entities, render| {
                // -- $$$FRK(TODO): only update dirty
                for i in 0..entity_model.models.len() {
                    let model_handle : entity_model::SHandle = i;

                    let entity_handle = entity_model.get_entity(model_handle);
                    if entities.get_location_update_frame(entity_handle) != game_context.cur_frame {
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
        game_context.data_bucket.get::<entity_animation::SBucket>()
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
        let imgui_draw_data = frame_context.imgui_ui.take().expect("this is where we take it").render();

        game_context.data_bucket.get::<render::SRender>()
            .and::<SEntityBucket>()
            .and::<entity_animation::SBucket>()
            .and::<entity_model::SBucket>()
            .and::<camera::SDebugFPCamera>()
            .with_mmmcc(|render, entities, entity_animation, entity_model, camera| {
                let view_matrix = camera.world_to_view_matrix();

                let render_result = render.render_frame(&mut window, &view_matrix, entities, entity_animation, entity_model, Some(&imgui_draw_data));
                match render_result {
                    Ok(_) => {},
                    Err(e) => {
                        println!("ERROR: render failed with error '{}'", e);
                        panic!();
                    },
                }
            });

        game_context.end_frame(frame_context);
        game_context.cur_frame += 1;

        // -- $$$FRK(TODO): framerate is uncapped

        game_context.data_bucket.get::<input::SInput>()
            .with_mut(|input| {
                input.mouse_dx = 0;
                input.mouse_dy = 0;

                input.mouse_cursor_pos_screen = winapi.rawwinapi().get_cursor_pos();
                input.mouse_cursor_pos_window = window.mouse_pos(&winapi.rawwinapi());

                let io = imgui_ctxt.io_mut(); // for filling out io state
                io.mouse_pos = [input.mouse_cursor_pos_window[0] as f32, input.mouse_cursor_pos_window[1] as f32];

                let mut input_handler = input.frame(io);
                loop {
                    let msg = window.pollmessage();
                    match msg {
                        None => break,
                        Some(m) => match m {
                            safewindows::EMsgType::Paint => {
                                //println!("Paint!");
                                window.dummyrepaint();
                            }
                            safewindows::EMsgType::KeyDown { key } => {
                                input_handler.handle_key_down_up(key, true);
                            },
                            safewindows::EMsgType::KeyUp { key } => {
                                input_handler.handle_key_down_up(key, false);
                            },
                            safewindows::EMsgType::LButtonDown{ .. } => {
                                input_handler.handle_lmouse_down_up(true);
                            },
                            safewindows::EMsgType::LButtonUp{ .. } => {
                                input_handler.handle_lmouse_down_up(false);
                            },
                            safewindows::EMsgType::MButtonDown{ .. } => {
                                input_handler.handle_mmouse_down_up(true);
                            },
                            safewindows::EMsgType::MButtonUp{ .. } => {
                                input_handler.handle_mmouse_down_up(false);
                            },
                            safewindows::EMsgType::Input{ raw_input } => {
                                if let safewindows::rawinput::ERawInputData::Mouse{data} = raw_input.data {
                                    input_handler.handle_mouse_move(data.last_x, data.last_y);
                                }
                            },
                            safewindows::EMsgType::Size => {
                                //println!("Size");
                                let rect: safewindows::SRect = window.raw().getclientrect().unwrap();
                                let newwidth = rect.right - rect.left;
                                let newheight = rect.bottom - rect.top;

                                game_context.data_bucket.get_renderer().with_mut(|render: &mut render::SRender| {
                                    render.resize_window(&mut window, newwidth, newheight)
                                }).unwrap();
                            }
                            safewindows::EMsgType::Invalid => (),
                        },
                    }
                }

                // -- display size might have changed
                io.display_size = [window.width() as f32, window.height() as f32];
                input.mouse_cursor_pos_window = window.mouse_pos(&winapi.rawwinapi());
            });

        drop(frame_linear_allocator);
        frame_linear_allocator_helper.reset();

        // -- increase frame time for testing
        //std::thread::sleep(std::time::Duration::from_millis(111));
    }

    // -- wait for all commands to clear
    game_context.data_bucket.get_renderer().with_mut(|render: &mut render::SRender| {
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
