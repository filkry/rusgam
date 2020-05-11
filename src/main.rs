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
mod input;
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
use glm::{Vec3, Vec4, Mat4};

use allocate::{STACK_ALLOCATOR, SMemVec};
use collections::{SPoolHandle};
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

#[derive(PartialEq)]
enum EEditMode {
    None,
    Translation,
}

impl EMode {
    pub fn toggle(&mut self) {
        match self {
            Self::Play => { *self = Self::Edit },
            Self::Edit => { *self = Self::Play },
        }
    }
}

// -- transform, camera pos, and camera forward must be in the same space
fn scale_to_fixed_screen_size(
    transform: &mut STransform,
    pct_of_near_plane_for_one_unit: f32,
    fovy: f32,
    znear: f32,
    window_width: u32,
    window_height: u32,
    camera_pos: &Vec3,
    camera_forward: &Vec3,
) {
    let fovx = utils::fovx(fovy, window_width, window_height);

    let to_fixed = transform.t - camera_pos;
    let dist = glm::length(&to_fixed);

    let angle_from_forward = glm::angle(&to_fixed, &camera_forward);
    let proj_dist = znear / (angle_from_forward).cos();

    // -- the whole idea of this code is to build a ratio of the similar
    // -- triangle from the object in world space to the amount of space
    // -- 1 unit will take up on the near plane projection, then scale it
    // -- so that space is constant
    let proj_ratio = proj_dist / dist;

    let unit_in_proj_space = 1.0 * proj_ratio;

    let total_proj_space = 2.0 * znear * (fovx / 2.0).tan();
    let desired_proj_space = total_proj_space * pct_of_near_plane_for_one_unit;

    let scale = desired_proj_space / unit_in_proj_space;

    transform.s = scale;
}

fn cursor_ray_world(
    mouse_pos: [i32; 2],
    render: &render::SRender,
    window: &n12::SD3D12Window,
    camera: &camera::SCamera,
) -> utils::SRay {
    let (x_pos, y_pos) = (mouse_pos[0], mouse_pos[1]);

    //println!("Left button down: {}, {}", x_pos, y_pos);

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

    //println!("to_z_near_camera_space: {:?}", to_z_near_camera_space);

    let world_to_view = camera.world_to_view_matrix();
    let view_to_world = glm::inverse(&world_to_view);

    let to_z_near_world_space = view_to_world * utils::vec3_to_homogenous(&to_z_near_camera_space, 0.0);

    utils::SRay{
        origin: camera.pos_world,
        dir: to_z_near_world_space.xyz(),
    }
}

fn world_pos_to_screen_pos(
    world_pos: &Vec3,
    view_matrix: &Mat4,
    window: &n12::SD3D12Window,
    render: &render::SRender,
) -> Vec3 {
    let perspective_matrix = {
        let aspect = (window.width() as f32) / (window.height() as f32);
        let zfar = 100.0;

        //SMat44::new_perspective(aspect, fovy, znear, zfar)
        glm::perspective_lh_zo(aspect, render.fovy(), render.znear(), zfar)
    };

    let view_perspective_matrix = perspective_matrix * view_matrix;

    let pos_clip_space = view_perspective_matrix * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    let pos_ndc = pos_clip_space / pos_clip_space.w;

    let width_f32 = window.width() as f32;
    let height_f32 = window.height() as f32;

    let screen_space = Vec3::new(
        ((pos_ndc.x + 1.0) / 2.0) * width_f32,
        ((-pos_ndc.y + 1.0) / 2.0) * height_f32,
        0.0, // not valid
    );

    screen_space
}

fn pos_on_screen_space_line_to_world(
    world_line_p0: &Vec3,
    world_line_p1: &Vec3,
    screen_space_pos: [i32; 2],
    view_matrix: &Mat4,
    window: &n12::SD3D12Window,
    render: &render::SRender,
) -> Vec3 {
    // -- how to move with translation widget:
    // + create two very distant points from the widget in world space on the translation axis
    // + get those points in screen space
    // + find closest point on that line to cursor
    // + project closest point in world space
    // + move object to that position (figure out offset later)

    //println!("Line p0 : {:?}", line_p0);
    //println!("Line p1 : {:?}", line_p1);

    let perspective_matrix = {
        let aspect = (window.width() as f32) / (window.height() as f32);
        let zfar = 100.0;

        //SMat44::new_perspective(aspect, fovy, znear, zfar)
        glm::perspective_lh_zo(aspect, render.fovy(), render.znear(), zfar)
    };

    let view_perspective_matrix = perspective_matrix * view_matrix;

    let mut line_p0_clip_space = view_perspective_matrix * Vec4::new(world_line_p0.x, world_line_p0.y, world_line_p0.z, 1.0);
    let mut line_p1_clip_space = view_perspective_matrix * Vec4::new(world_line_p1.x, world_line_p1.y, world_line_p1.z, 1.0);
    //println!("Line p0 clip space: {:?}", line_p0_clip_space);
    //println!("Line p1 clip space: {:?}", line_p1_clip_space);

    let line_p0_w = line_p0_clip_space.w;
    let line_p1_w = line_p1_clip_space.w;
    line_p0_clip_space /= line_p0_w;
    line_p1_clip_space /= line_p1_w;

    //println!("Line p0 clip space NORM: {:?}", line_p0_clip_space);
    //println!("Line p1 clip space NORM: {:?}", line_p1_clip_space);

    let width_f32 = window.width() as f32;
    let height_f32 = window.height() as f32;

    let line_p0_screen_space = Vec3::new(
        ((line_p0_clip_space.x + 1.0) / 2.0) * width_f32,
        ((-line_p0_clip_space.y + 1.0) / 2.0) * height_f32,
        0.0, // not valid
    );
    let line_p1_screen_space = Vec3::new(
        ((line_p1_clip_space.x + 1.0) / 2.0) * width_f32,
        ((-line_p1_clip_space.y + 1.0) / 2.0) * height_f32,
        0.0, // not valid
    );

    let line_screen_space = line_p1_screen_space - line_p0_screen_space;

    println!("Line p0 screen space: {:?}", line_p0_screen_space);
    println!("Line p1 screen space: {:?}", line_p1_screen_space);

    // -- mouse is thought of as on the znear plane, so 0.0
    let mouse_pos_v = Vec3::new(screen_space_pos[0] as f32, screen_space_pos[1] as f32, 0.0);

    let (closest_pos_screen_space, _) = utils::closest_point_on_line(&line_p0_screen_space, &line_p1_screen_space, &mouse_pos_v);
    println!("closest pos screen space: {:?}", closest_pos_screen_space);

    let closest_pos_clip_ndc = Vec3::new(
        (closest_pos_screen_space.x / width_f32) * 2.0 - 1.0,
        -((closest_pos_screen_space.y / height_f32) * 2.0 - 1.0),
        0.0, // not valid
    );

    // the basic idea: we want to find a point P  on a line, such that for
    // (1) P' = view_perspective_matrix * P,
    // (3) P'.x / P'.w = cursor_ndc.x AND P'.y / P'.w = cursor_ndc.y
    //
    // P must be on the line, so (2) P = line_p0 + t * d, where d = line_p1 - line_p0
    // we have only one unknown 't', so we can simplify to a single equation in x or y
    //
    // If you simplify for just x, you get:
    // P'.x = dot(view_perspective_matrix.row_0, P);
    // P'.w = dot(view_perspective_matrix.row_3, P);
    //
    // combined with (2) and simplified:
    // P'.x = dot(row_0, line_p0) + t * dot(row_0, d)
    // P'.w = dot(row_3, line_p0) + t * dot(row_3, d)
    //
    // combined with (3) gives you:
    // t = cursor_ndc.x * dot(row_3, line_p0) - dot(row_0, line_p0)
    //     ------------------------------------------------------------------
    //     dot(row_0, d) - cursor_ndc.x * dot(row_3, d)

    // -- re-used values
    let d = world_line_p1 - world_line_p0;
    let d_vec4 = Vec4::new(d.x, d.y, d.z, 0.0);
    let line_p0_vec4 = Vec4::new(world_line_p0.x, world_line_p0.y, world_line_p0.z, 1.0);

    let row_0 = glm::row(&view_perspective_matrix, 0);
    let row_1 = glm::row(&view_perspective_matrix, 1);
    let row_3 = glm::row(&view_perspective_matrix, 3);
    let row_0_dot_p0 = glm::dot(&row_0, &line_p0_vec4);
    let row_1_dot_p0 = glm::dot(&row_1, &line_p0_vec4);
    let row_3_dot_p0 = glm::dot(&row_3, &line_p0_vec4);
    let row_0_dot_d = glm::dot(&row_0, &d_vec4);
    let row_1_dot_d = glm::dot(&row_1, &d_vec4);
    let row_3_dot_d = glm::dot(&row_3, &d_vec4);

    let t = {
        if line_screen_space.x.abs() > line_screen_space.y.abs() {
            let t_numer = closest_pos_clip_ndc.x * row_3_dot_p0 - row_0_dot_p0;
            let t_denom = row_0_dot_d - closest_pos_clip_ndc.x * row_3_dot_d;
            t_numer / t_denom
        }
        else {
            let t_numer = closest_pos_clip_ndc.y * row_3_dot_p0 - row_1_dot_p0;
            let t_denom = row_1_dot_d - closest_pos_clip_ndc.y * row_3_dot_d;
            t_numer / t_denom
        }
    };

    let closest_pos_world_space = world_line_p0 + t * d;

    closest_pos_world_space.xyz()
}

fn main_d3d12() -> Result<(), &'static str> {
    render::compile_shaders_if_changed();

    let winapi = rustywindows::SWinAPI::create();

    let mut imgui_ctxt = imgui::Context::create();
    input::setup_imgui_key_map(imgui_ctxt.io_mut());

    let mut render = render::SRender::new(&winapi, &mut imgui_ctxt)?;

    // -- setup window
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();
    let mut window = render.create_window(&windowclass, "rusgam", 1600, 900)?;

    window.init_render_target_views(render.device())?;
    window.show();

    let translation_widget = render.new_model("assets/arrow_widget.obj", 1.0, false)?;
    let mut translation_start_pos : Vec3 = glm::zero();
    let mut translation_widget_transform = STransform::default();
    let mut translation_mouse_offset = [0; 2];

    let mut entities = entity::SEntityBucket::new(67485, 16);
    let rotating_entity = entities.create_entity()?;
    let debug_entity = entities.create_entity()?;
    {
        // -- set up entities
        let ent2 = entities.create_entity()?;
        let ent3 = entities.create_entity()?;
        let room = entities.create_entity()?;

        let model1 = render.new_model("assets/first_test_asset.obj", 1.0, true)?;
        let model3 = render.new_model("assets/test_untextured_flat_colour_cube.obj", 1.0, true)?;
        let room_model = render.new_model("assets/test_open_room.obj", 1.0, true)?;
        let mut debug_model = render.new_model("assets/debug_icosphere.obj", 1.0, true)?;
        debug_model.set_pickable(false);
        //let fixed_size_model = SModel::new_from_obj("assets/test_untextured_flat_colour_cube.obj", &device, &mut copycommandpool, &mut directcommandpool, &srv_heap, true, 1.0)?;

        entities.set_entity_debug_name(rotating_entity, "tst_rotating");
        entities.set_entity_debug_name(ent2, "tst_textured_cube");
        entities.set_entity_debug_name(ent3, "tst_coloured_cube");
        entities.set_entity_debug_name(room, "tst_room");

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

    let start_time = winapi.curtimemicroseconds();
    let rot_axis = Vec3::new(0.0, 1.0, 0.0);

    let mut camera = camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0));

    let mut input = input::SInput::new();

    let mut mode = EMode::Edit;
    let mut edit_mode = EEditMode::None;

    let mut last_ray_hit_pos = Vec3::new(0.0, 0.0, 0.0);
    let mut last_picked_entity : Option<SPoolHandle> = None;

    let mut show_imgui_demo_window = false;

    while !input.q_down {
        if input.tilde_edge.down() {
            mode.toggle();
        }

        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;
        let dts = (dt as f32) / 1_000_000.0;

        let total_time = curframetime - start_time;

        let mouse_pos = window.mouse_pos(&winapi.rawwinapi());

        // -- update
        let cur_angle = ((total_time as f32) / 1_000_000.0) * (3.14159 / 4.0);
        entities.set_entity_location(rotating_entity, STransform::new_rotation(&glm::quat_angle_axis(cur_angle, &rot_axis)));
        entities.set_entity_location(debug_entity, STransform::new_translation(&last_ray_hit_pos));

        //let mut fixed_size_model_xform = STransform::new_translation(&glm::Vec3::new(0.0, 5.0, 0.0));

        let mut can_rotate_camera = false;
        if let EMode::Play = mode {
            can_rotate_camera = true;
        }
        else if input.middle_mouse_down {
            can_rotate_camera = true;
        }
        camera.update_from_input(&input, dts, can_rotate_camera);

        input.mouse_dx = 0;
        input.mouse_dy = 0;
        let view_matrix = camera.world_to_view_matrix();
        let cursor_ray = cursor_ray_world(mouse_pos, &render, &window, &camera);

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);

        // -- update translation widget
        if mode == EMode::Edit {
            if let Some(e) = last_picked_entity {
                translation_widget_transform = entities.get_entity_location(e);
                //println!("Set translation widget: {:?}", translation_widget_transform.t);
                scale_to_fixed_screen_size(&mut translation_widget_transform, 0.02, render.fovy(), render.znear(), window.width(), window.height(), &camera.pos_world, &camera.forward_world());
            }
        }

        // -- check if the user clicked a translation widget
        let mut hit_translation_widget = false;
        if input.left_mouse_edge.down() && !imgui_ctxt.io().want_capture_mouse {
            if let Some(_) = render.ray_intersects(&translation_widget, &cursor_ray.origin, &cursor_ray.dir, &translation_widget_transform) {
                let e = last_picked_entity.expect("shouldn't be able to translate without entity picked.");

                translation_start_pos = translation_widget_transform.t;
                println!("Hit translation widget!");
                hit_translation_widget = true;
                edit_mode = EEditMode::Translation;

                let e_pos = entities.get_entity_location(e).t;
                let e_pos_screen = world_pos_to_screen_pos(&e_pos, &view_matrix, &window, &render);

                translation_mouse_offset = [(e_pos_screen.x as i32) - mouse_pos[0],
                                            (e_pos_screen.y as i32) - mouse_pos[1]];
            }
        }

        if mode == EMode::Edit && edit_mode == EEditMode::Translation {
            if !input.left_mouse_down {
                edit_mode = EEditMode::None;
            }
            else {
                assert!(last_picked_entity.is_some(), "translating but no entity!");

                let line_p0 = translation_start_pos + Vec3::new(0.0, 0.0, -1.0);
                let line_p1 = translation_start_pos + Vec3::new(0.0, 0.0, 1.0);

                let offset_mouse_pos = [mouse_pos[0] + translation_mouse_offset[0],
                                        mouse_pos[1] + translation_mouse_offset[1]];

                let new_world_pos = pos_on_screen_space_line_to_world(
                    &line_p0,
                    &line_p1,
                    offset_mouse_pos,
                    &view_matrix,
                    &window,
                    &render,
                );

                entities.set_entity_location(
                    last_picked_entity.expect(""),
                    STransform::new_translation(&new_world_pos)
                );
            }
        }

        // -- render world
        STACK_ALLOCATOR.with(|sa| -> Result<(), &'static str> {
            let (entities, model_xforms, models) = entities.build_render_data(sa);
            render.render(&mut window, &view_matrix, models.as_slice(), model_xforms.as_slice())?;

            if input.left_mouse_edge.down() && !imgui_ctxt.io().want_capture_mouse && !hit_translation_widget {

                let mut min_t = std::f32::MAX;
                let mut min_model_i = None;
                let mut min_pos = Vec3::new(0.0, 0.0, 0.0);

                for modeli in 0..models.len() {
                    if let Some(t) = render.ray_intersects(&models[modeli], &cursor_ray.origin, &cursor_ray.dir, &model_xforms[modeli]) {
                        assert!(t > 0.0);
                        if t < min_t {
                            min_t = t;
                            min_model_i = Some(modeli);
                            min_pos = camera.pos_world + t * cursor_ray.dir;
                        }
                    }
                }

                if let Some(modeli) = min_model_i {
                    //println!("Hit model {} at pos {}, {}, {}", modeli, min_pos.x, min_pos.y, min_pos.z);
                    last_ray_hit_pos = min_pos;
                    last_picked_entity = Some(entities[modeli]);
                }
                else {
                    last_picked_entity = None;
                }
            }

            Ok(())
        })?;

        // -- render non-depth-tested things
        STACK_ALLOCATOR.with(|sa| -> Result<(), &'static str> {
            let mut no_depth_models = SMemVec::new(sa, 32, 0)?;
            let mut no_depth_transforms = SMemVec::new(sa, 32, 0)?;

            if let Some(_) = last_picked_entity {
                no_depth_models.push(translation_widget);
                no_depth_transforms.push(translation_widget_transform);
            }

            if no_depth_models.len() > 0 {
                render.render_no_depth(&mut window, &view_matrix, no_depth_models.as_slice(), no_depth_transforms.as_slice())?;
            }

            Ok(())
        })?;

        // -- render IMGUI
        // -- set up imgui IO
        if let EMode::Edit = mode {

            let io = imgui_ctxt.io_mut();
            io.display_size = [window.width() as f32, window.height() as f32];

            let imgui_ui = imgui_ctxt.frame();
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
            });

            if let Some(e) = last_picked_entity {
                entities.show_imgui_window(e, &imgui_ui);
            }

            let imgui_draw_data = imgui_ui.render();

            render.setup_imgui_draw_data_resources(&window, &imgui_draw_data)?;
            render.render_imgui(&mut window, imgui_draw_data)?;
        }

        render.present(&mut window)?;

        lastframetime = curframetime;
        _framecount += 1;


        // -- $$$FRK(TODO): framerate is uncapped

        let io = imgui_ctxt.io_mut(); // for filling out io state
        io.mouse_pos = [mouse_pos[0] as f32, mouse_pos[1] as f32];

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

    // -- find out what we leaked
    drop(render);
    //let debug_interface = t12::SDXGIDebugInterface::new()?;
    //debug_interface.report_live_objects();

    Ok(())
}

fn debug_test() {}

fn main() {
    debug_test();

    main_d3d12().unwrap();
}
