extern crate arrayvec;
//extern crate nalgebra_glm as glm;
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
mod math;
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
use animation::{SAnimationLoader};
use allocate::{SYSTEM_ALLOCATOR, SAllocator};
use entity::{SEntityBucket};
use game_context::{SGameContext, SFrameContext};
use math::{Vec3, Vec4};
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

    entity_animation::update_animation(game_context, frame_context);
    update_entity_bvh_entries(game_context, frame_context);

    // -- debug updates
    debug_ui::update_debug_main_menu(game_context, frame_context);
    debug_ui::update_debug_entity_menu(game_context, frame_context);
    debug_ui::update_draw_entity_bvh(game_context, frame_context);

    frame_context.finalize_ui();

    render::update_render_frame(game_context, frame_context);

    Ok(())
}

pub fn update_entity_bvh_entries(game_context: &SGameContext, _frame_context: &SFrameContext) {
    game_context.data_bucket.get::<bvh::STree<entity::SEntityHandle>>()
        .and::<entity_model::SBucket>()
        .and::<SEntityBucket>()
        .and::<render::SRender>()
        .with_mmcc(|bvh, entity_model, entities, render| {
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
                    let new_bvh_handle = bvh.insert(entity_handle, &transformed_aabb, None)
                        .expect("out of BVH pool space");
                    entity_model.set_bvh_entry(model_handle, new_bvh_handle);
                }
            }
        });
}

fn main_d3d12(d3d_debug: bool) -> Result<(), &'static str> {
    render::compile_shaders_if_changed(d3d_debug);

    let winapi = rustywindows::SWinAPI::create();

    let mut imgui_ctxt = imgui::Context::create();

    input::setup_imgui_key_map(imgui_ctxt.io_mut());

    let mut render = render::SRender::new(&winapi, &mut imgui_ctxt, d3d_debug)?;

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

    let mut game_context = SGameContext::new(&winapi, window);

    game_context.data_bucket.add(SEntityBucket::new(16));
    game_context.data_bucket.add(SAnimationLoader::new(SYSTEM_ALLOCATOR(), 64));
    game_context.data_bucket.add(game_mode::SGameMode::new(&mut render));
    game_context.data_bucket.add(render);
    game_context.data_bucket.add(entity_model::SBucket::new(&SYSTEM_ALLOCATOR(), 1024)?);
    game_context.data_bucket.add(entity_animation::SBucket::new(&SYSTEM_ALLOCATOR(), 1024)?);
    game_context.data_bucket.add(bvh::STree::new());
    game_context.data_bucket.add(camera::SDebugFPCamera::new(Vec3::new(0.0, 0.0, -10.0)));
    game_context.data_bucket.add(input::SInput::new());
    game_context.data_bucket.add(gjk::SGJKDebug::new(&game_context.data_bucket));

    entitytypes::testtexturedcubeentity::create(
        &game_context, Some("tst_rotating"),
        STransform::new_translation(&Vec3::new(0.0, 0.0, 0.0)))?;
    entitytypes::testtexturedcubeentity::create(
        &game_context, Some("tst_textured_cube"),
        STransform::new_translation(&Vec3::new(3.0, 0.0, 0.0)))?;
    entitytypes::flatshadedcubeentity::create(
        &game_context, Some("tst_coloured_cube"), Some(Vec4::new(1.0, 0.0, 0.0, 0.9)),
        STransform::new_translation(&Vec3::new(0.0, 2.0, 0.0)))?;
    entitytypes::testopenroomentity::create(
        &game_context, Some("tst_room"),
        STransform::new_translation(&Vec3::new(0.0, -2.0, 0.0)))?;
    let skinned_entity = entitytypes::tstskinnedentity::create(
        &game_context, Some("tst_skinned_entity"), Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
        STransform::new_translation(&Vec3::new(-3.0, 2.0, 0.0)))?;

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
        let mut frame_context = game_context.start_frame(
            &winapi,
            &mut imgui_ctxt,
            frame_linear_allocator,
        );

        update_frame(&game_context, &mut frame_context)?;

        // -- flip swap chain
        game_context.data_bucket.get::<render::SRender>()
            .build()
            .with_mut(|render| {
                render.present(&mut game_context.window)
            })?;

        game_context.end_frame(frame_context);
        game_context.cur_frame += 1;

        // -- $$$FRK(TODO): framerate is uncapped

        game_context.data_bucket.get::<input::SInput>()
            .build()
            .with_mut(|input| {
                input.mouse_dx = 0;
                input.mouse_dy = 0;

                input.mouse_cursor_pos_screen = winapi.rawwinapi().get_cursor_pos();
                input.mouse_cursor_pos_window = game_context.window.mouse_pos(&winapi.rawwinapi());

                let io = imgui_ctxt.io_mut(); // for filling out io state
                io.mouse_pos = [input.mouse_cursor_pos_window[0] as f32, input.mouse_cursor_pos_window[1] as f32];

                let mut input_handler = input.frame(io);
                loop {
                    let msg = game_context.window.pollmessage();
                    match msg {
                        None => break,
                        Some(m) => match m {
                            safewindows::EMsgType::Paint => {
                                //println!("Paint!");
                                game_context.window.dummyrepaint();
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
                                let rect: safewindows::SRect = game_context.window.raw().getclientrect().unwrap();
                                let newwidth = rect.right - rect.left;
                                let newheight = rect.bottom - rect.top;

                                game_context.data_bucket.get_renderer().build().with_mut(|render: &mut render::SRender| {
                                    render.resize_window(&mut game_context.window, newwidth, newheight)
                                }).unwrap();
                            }
                            safewindows::EMsgType::Invalid => (),
                        },
                    }
                }

                // -- display size might have changed
                io.display_size = [game_context.window.width() as f32, game_context.window.height() as f32];
                input.mouse_cursor_pos_window = game_context.window.mouse_pos(&winapi.rawwinapi());
            });

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

fn main() {
    let mut d3d_debug = false;
    let args : Vec::<String> = std::env::args().collect();
    for arg in args {
        if arg.trim() == "d3d-debug" {
            d3d_debug = true;
        }
    }

    use std::panic;
    panic::set_hook(Box::new(|_| {
        safewindows::break_if_debugging();
    }));

    let result = main_d3d12(d3d_debug);
    if let Err(e) = result {
        println!("Aborted with error: {:?}", e);
    }
}
