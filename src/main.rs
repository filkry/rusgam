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

//mod math;
#[macro_use]
mod safewindows;
mod allocate;
mod collections;
mod directxgraphicssamples;
mod entity;
mod niced3d12;
mod rustywindows;
mod typeyd3d12;
mod utils;
mod enumflags;
mod camera;
mod model;
mod render;
mod shadowmapping;

// -- std includes
//use std::cell::RefCell;
//use std::mem::size_of;
//use std::io::Write;
//use std::rc::Rc;
//use std::ops::{Deref, DerefMut};

// -- crate includes
//use arrayvec::{ArrayVec};
//use serde::{Serialize, Deserialize};
use glm::{Vec3/*, Mat4*/};

use allocate::{STACK_ALLOCATOR};
use niced3d12 as n12;
use typeyd3d12 as t12;
//use allocate::{SMemVec, STACK_ALLOCATOR};
use utils::{STransform};
//use model::{SModel, SMeshLoader, STextureLoader};

pub struct SInput {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    space: bool,
    c: bool,
    mouse_dx: i32,
    mouse_dy: i32,
}

fn main_d3d12() -> Result<(), &'static str> {
    render::compile_shaders_if_changed();

    let winapi = rustywindows::SWinAPI::create();

    let mut render = render::SRender::new(&winapi)?;

    // -- setup window
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut window = render.create_window(&windowclass, "rusgam", 800, 600)?;

    window.init_render_target_views(render.device())?;
    window.show();

    let mut imgui_ctxt = imgui::Context::create();

    // -- set up imgui
    {
        let font_size = 13.0 as f32;
        imgui_ctxt.fonts().add_font(&[
            imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..imgui::FontConfig::default()
                }),
            },
            imgui::FontSource::TtfData {
                data: include_bytes!("../assets/mplus-1p-regular.ttf"),
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    rasterizer_multiply: 1.75,
                    glyph_ranges: imgui::FontGlyphRanges::japanese(),
                    ..imgui::FontConfig::default()
                }),
            },
        ]);

        imgui_ctxt.fonts().build_rgba32_texture();
    }

    let mut entities = entity::SEntityBucket::new(67485, 16);
    let rotating_entity = entities.create_entity()?;
    let debug_entity = entities.create_entity()?;
    {
        // -- set up entities
        let ent2 = entities.create_entity()?;
        let ent3 = entities.create_entity()?;
        let room = entities.create_entity()?;

        let model1 = render.new_model("assets/first_test_asset.obj", 1.0)?;
        let model3 = render.new_model("assets/test_untextured_flat_colour_cube.obj", 1.0)?;
        let room_model = render.new_model("assets/test_open_room.obj", 1.0)?;
        let debug_model = render.new_model("assets/debug_icosphere.obj", 1.0)?;
        //let fixed_size_model = SModel::new_from_obj("assets/test_untextured_flat_colour_cube.obj", &device, &mut copycommandpool, &mut directcommandpool, &srv_heap, true, 1.0)?;
        //let translation_widget = SModel::new_from_obj("assets/arrow_widget.obj", &mut mesh_loader, &mut texture_loader, 0.8)?;

        entities.set_entity_location(ent2, STransform::new_translation(&glm::Vec3::new(3.0, 0.0, 0.0)));
        entities.set_entity_location(ent3, STransform::new_translation(&glm::Vec3::new(0.0, 2.0, 0.0)));
        entities.set_entity_location(room, STransform::new_translation(&glm::Vec3::new(0.0, -2.0, 0.0)));

        entities.set_entity_model(rotating_entity, model1.clone());
        entities.set_entity_model(ent2, model1.clone());
        entities.set_entity_model(ent3, model3);
        entities.set_entity_model(room, room_model);
        entities.set_entity_model(debug_entity, debug_model);
    }

    // -- update loop

    let mut _framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut shouldquit = false;

    let start_time = winapi.curtimemicroseconds();
    let rot_axis = Vec3::new(0.0, 1.0, 0.0);

    let mut camera = camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0));

    let mut input = SInput{
        w: false,
        a: false,
        s: false,
        d: false,
        space: false,
        c: false,

        mouse_dx: 0,
        mouse_dy: 0,
    };

    let mut last_ray_hit_pos = Vec3::new(0.0, 0.0, 0.0);

    while !shouldquit {
        // -- set up imgui IO
        {
            let io = imgui_ctxt.io_mut();
            io.display_size = [window.width() as f32, window.height() as f32];
        }

        let mut imgui_ui = imgui_ctxt.frame();

        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;
        let dts = (dt as f32) / 1_000_000.0;

        let total_time = curframetime - start_time;

        // -- update
        let cur_angle = ((total_time as f32) / 1_000_000.0) * (3.14159 / 4.0);
        entities.set_entity_location(rotating_entity, STransform::new_rotation(&glm::quat_angle_axis(cur_angle, &rot_axis)));
        entities.set_entity_location(debug_entity, STransform::new_translation(&last_ray_hit_pos));

        let mut fixed_size_model_xform = STransform::new_translation(&glm::Vec3::new(0.0, 5.0, 0.0));

        {
            let fovx = utils::fovx(render.fovy(), window.width(), window.height());

            let to_fixed = fixed_size_model_xform.t - camera.pos_world;
            let dist = glm::length(&to_fixed);

            let angle_from_forward = glm::angle(&to_fixed, &camera.forward_world());
            let proj_dist = render.znear() / (angle_from_forward).cos();

            // -- the whole idea of this code is to build a ratio of the similar
            // -- triangle from the object in world space to the amount of space
            // -- 1 unit will take up on the near plane projection, then scale it
            // -- so that space is constant
            let proj_ratio = proj_dist / dist;

            let unit_in_proj_space = 1.0 * proj_ratio;

            let total_proj_space = 2.0 * render.znear() * (fovx / 2.0).tan();
            let desired_proj_space = total_proj_space / 10.0;

            let scale = desired_proj_space / unit_in_proj_space;

            fixed_size_model_xform.s = scale;
        }

        camera.update_from_input(&input, dts);
        input.mouse_dx = 0;
        input.mouse_dy = 0;
        let view_matrix = camera.world_to_view_matrix();

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);
        STACK_ALLOCATOR.with(|sa| -> Result<(), &'static str> {
            let (model_xforms, models) = entities.build_render_data(sa);
            render.render(&mut window, &view_matrix, models.as_slice(), model_xforms.as_slice())
        })?;

        let mut opened = true;
        imgui_ui.show_demo_window(&mut opened);

        let imgui_draw_data = imgui_ui.render();

        lastframetime = curframetime;
        _framecount += 1;

        // -- $$$FRK(TODO): framerate is uncapped

        loop {
            let msg = window.pollmessage();
            match msg {
                None => break,
                Some(m) => match m {
                    safewindows::EMsgType::Paint => {
                        //println!("Paint!");
                        window.dummyrepaint();
                    }
                    safewindows::EMsgType::KeyDown { key } => match key {
                        safewindows::EKey::Q => {
                            shouldquit = true;
                            //println!("Q keydown");
                        }
                        safewindows::EKey::W => input.w = true,
                        safewindows::EKey::A => input.a = true,
                        safewindows::EKey::S => input.s = true,
                        safewindows::EKey::D => input.d = true,
                        safewindows::EKey::Space => input.space = true,
                        safewindows::EKey::C => input.c = true,
                        _ => (),
                    },
                    safewindows::EMsgType::KeyUp { key } => match key {
                        safewindows::EKey::W => input.w = false,
                        safewindows::EKey::A => input.a = false,
                        safewindows::EKey::S => input.s = false,
                        safewindows::EKey::D => input.d = false,
                        safewindows::EKey::Space => input.space = false,
                        safewindows::EKey::C => input.c = false,
                        _ => (),
                    },
                    safewindows::EMsgType::LButtonDown{ x_pos, y_pos } => {
                        /*
                        println!("Left button down: {}, {}", x_pos, y_pos);

                        let half_camera_near_clip_height = (render.fovy()/2.0).tan() * render.znear();
                        let half_camera_near_clip_width = ((window.width() as f32) / (window.height() as f32)) * half_camera_near_clip_height;

                        let near_clip_top_left_camera_space = Vec3::new(-half_camera_near_clip_width, half_camera_near_clip_height, render.znear());
                        let near_clip_deltax_camera_space = Vec3::new(2.0 * half_camera_near_clip_width, 0.0, 0.0);
                        let near_clip_deltay_camera_space = Vec3::new(0.0, -2.0 * half_camera_near_clip_height, 0.0);

                        let pct_width = (x_pos as f32) / (window.width() as f32);
                        let pct_height = (y_pos as f32) / (window.height() as f32);

                        let to_z_near_camera_space = near_clip_top_left_camera_space +
                            pct_width * near_clip_deltax_camera_space +
                            pct_height * near_clip_deltay_camera_space;

                        println!("to_z_near_camera_space: {:?}", to_z_near_camera_space);

                        let world_to_view = camera.world_to_view_matrix();
                        let view_to_world = glm::inverse(&world_to_view);

                        let to_z_near_world_space = view_to_world * utils::vec3_to_homogenous(&to_z_near_camera_space, 0.0);

                        let mut min_t = std::f32::MAX;
                        let mut min_model_i = None;
                        let mut min_pos = Vec3::new(0.0, 0.0, 0.0);

                        for modeli in 0..models.len() {
                            if let Some(t) = render.ray_intersects(&model, &camera.pos_world, &to_z_near_world_space.xyz(), model_xforms[modeli]) {
                                if t < min_t {
                                    min_t = t;
                                    min_model_i = Some(modeli);
                                    min_pos = camera.pos_world + t * to_z_near_world_space.xyz();
                                }
                            }
                        }

                        if let Some(modeli) = min_model_i {
                            println!("Hit model {}", modeli);
                            last_ray_hit_pos = min_pos;
                        }
                        */
                    },
                    safewindows::EMsgType::Input{ raw_input } => {
                        if let safewindows::rawinput::ERawInputData::Mouse{data} = raw_input.data {
                            //println!("Frame {}: Raw Mouse: {}, {}", _framecount, data.last_x, data.last_y);
                            input.mouse_dx = data.last_x;
                            input.mouse_dy = data.last_y;
                        }
                    },
                    safewindows::EMsgType::Size => {
                        //println!("Size");
                        let rect: safewindows::SRect = window.raw().getclientrect()?;
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        render.resize_window(&mut window, newwidth, newheight)?;
                    }
                    safewindows::EMsgType::Invalid => (),
                },
            }
        }

        // -- increase frame time for testing
        //std::thread::sleep(std::time::Duration::from_millis(111));
    }

    // -- wait for all commands to clear
    render.flush()?;

    Ok(())
}

fn debug_test() {}

fn main() {
    debug_test();

    main_d3d12().unwrap();
}
