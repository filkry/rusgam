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
use glm::{Vec3, Vec4};

use animation::{SAnimationLoader};
use allocate::{STACK_ALLOCATOR, SYSTEM_ALLOCATOR, SMemVec};
use editmode::{SEditModeContext, EEditMode};
use entity::{SEntityBucket};
use game_context::{SGameContext};
use niced3d12 as n12;
use typeyd3d12 as t12;
//use allocate::{SMemVec, STACK_ALLOCATOR};
use utils::{STransform};
//use model::{SModel, SMeshLoader, STextureLoader};

#[derive(PartialEq)]
enum EMode {
    Play,
    Edit,
}

impl EMode {
    pub fn toggle(&mut self, edit_mode: &mut EEditMode) {
        match self {
            Self::Play => {
                *self = Self::Edit;
                *edit_mode = EEditMode::None;
            },
            Self::Edit => {
                *self = Self::Play;
                *edit_mode = EEditMode::Translation;
            },
        }
    }
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

    window.init_render_target_views(render.device())?;
    window.show();

    let mut editmode_ctxt = SEditModeContext::new(&mut render).unwrap();

    let mut game_context = SGameContext{
        cur_frame: 0,
        data_bucket: databucket::SDataBucket::new(256, &SYSTEM_ALLOCATOR),
    };

    game_context.data_bucket.add(SEntityBucket::new(67485, 16));
    game_context.data_bucket.add(SAnimationLoader::new(&SYSTEM_ALLOCATOR, 64));
    game_context.data_bucket.add(render);
    game_context.data_bucket.add(entity_model::SBucket::new(&SYSTEM_ALLOCATOR, 1024)?);
    game_context.data_bucket.add(entity_animation::SBucket::new(&SYSTEM_ALLOCATOR, 1024)?);
    game_context.data_bucket.add(bvh::STree::new());

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

    // -- update loop
    let mut lastframetime = winapi.curtimemicroseconds();

    let start_time = winapi.curtimemicroseconds();
    let _rot_axis = Vec3::new(0.0, 1.0, 0.0);

    let mut camera = camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0));

    let mut input = input::SInput::new();

    let mut mode = EMode::Edit;
    let mut edit_mode = EEditMode::None;

    let mut draw_selected_bvh  = false;

    let mut show_imgui_demo_window = false;

    let mut gjk_debug = gjk::SGJKDebug::new(&game_context.data_bucket);

    while !input.q_down {
        // -- handle edit mode toggles
        if input.tilde_edge.down() {
            mode.toggle(&mut edit_mode);
        }

        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;
        let dts = (dt as f32) / 1_000_000.0;

        let _total_time = curframetime - start_time;
        let _total_time_seconds = (_total_time as f32) / 1_000_000.0;

        // -- update
        /*
        let cur_angle = _total_time_seconds * (3.14159 / 4.0);
        data_bucket.get_entities().unwrap().with_mut(|entities: &mut SEntityBucket| {
            entities.set_location(gc, rotating_entity, STransform::new_rotation(&glm::quat_angle_axis(cur_angle, &_rot_axis)));
        });
        */

        //let mut fixed_size_model_xform = STransform::new_translation(&glm::Vec3::new(0.0, 5.0, 0.0));

        let mut can_rotate_camera = false;
        if let EMode::Play = mode {
            can_rotate_camera = true;
        }
        else if input.middle_mouse_down {
            can_rotate_camera = true;
        }
        camera.update_from_input(&input, dts, can_rotate_camera);

        let editmode_input = game_context.data_bucket.get_renderer().with(|render: &render::SRender| {
            editmode::SEditModeInput::new_for_frame(&window, &winapi, &camera, &render, &imgui_ctxt)
        });

        input.mouse_dx = 0;
        input.mouse_dy = 0;
        let view_matrix = camera.world_to_view_matrix();

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);

        // update edit mode
        if mode == EMode::Edit {
            edit_mode = edit_mode.update(&game_context, &mut editmode_ctxt, &editmode_input, &input, &game_context.data_bucket);
        }

        // -- update IMGUI
        let io = imgui_ctxt.io_mut();
        io.display_size = [window.width() as f32, window.height() as f32];

        let imgui_ui = imgui_ctxt.frame();
        if let EMode::Edit = mode {

            if show_imgui_demo_window {
                let mut opened = true;
                imgui_ui.show_demo_window(&mut opened);
            }

            imgui_ui.main_menu_bar(|| {
                imgui_ui.menu(imgui::im_str!("Misc"), true, || {
                    if imgui::MenuItem::new(imgui::im_str!("Toggle Demo Window")).build(&imgui_ui) {
                        show_imgui_demo_window = !show_imgui_demo_window;
                    }
                });

                game_context.data_bucket.get_bvh().with(|bvh: &bvh::STree<entity::SEntityHandle>| {
                    bvh.imgui_menu(&imgui_ui, &mut draw_selected_bvh);
                });

                gjk_debug.imgui_menu(&imgui_ui, &game_context.data_bucket, editmode_ctxt.editing_entity(), Some(rotating_entity));

            });

            if let Some(e) = editmode_ctxt.editing_entity() {
                game_context.data_bucket.get_entities().with_mut(|entities: &mut SEntityBucket| {
                    entities.show_imgui_window(e, &imgui_ui);
                });
            }
        }

        // -- draw selected object's BVH heirarchy
        if draw_selected_bvh {
            if let Some(e) = editmode_ctxt.editing_entity() {
                STACK_ALLOCATOR.with(|sa| {
                    game_context.data_bucket.get::<render::SRender>()
                        .and::<entity_model::SBucket>()
                        .and::<bvh::STree<entity::SEntityHandle>>()
                        .with_mcc(|render, em, bvh| {
                            let model_handle = em.handle_for_entity(e).unwrap();

                            let mut aabbs = SMemVec::new(sa, 32, 0).unwrap();
                            bvh.get_bvh_heirarchy_for_entry(em.get_bvh_entry(model_handle).unwrap(), &mut aabbs);
                            for aabb in aabbs.as_slice() {
                                render.temp().draw_aabb(aabb, &Vec4::new(1.0, 0.0, 0.0, 1.0), true);
                            }
                        });
                });
            }
        }

        // -- draw selected object colliding/not with rotating_entity
        if let Some(e) = editmode_ctxt.editing_entity() {
            STACK_ALLOCATOR.with(|sa| {
                game_context.data_bucket.get::<render::SRender>()
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
                e_animation.update_joints(anim_loader, _total_time_seconds);
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
        let imgui_draw_data = imgui_ui.render();

        game_context.data_bucket.get::<render::SRender>()
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

        lastframetime = curframetime;

        game_context.cur_frame += 1;

        // -- $$$FRK(TODO): framerate is uncapped

        let io = imgui_ctxt.io_mut(); // for filling out io state
        io.mouse_pos = [editmode_input.mouse_window_pos[0] as f32, editmode_input.mouse_window_pos[1] as f32];

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
                        let rect: safewindows::SRect = window.raw().getclientrect()?;
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        game_context.data_bucket.get_renderer().with_mut(|render: &mut render::SRender| {
                            render.resize_window(&mut window, newwidth, newheight)
                        })?;
                    }
                    safewindows::EMsgType::Invalid => (),
                },
            }
        }

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
