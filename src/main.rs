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
mod bvh;
mod collections;
mod databucket;
mod directxgraphicssamples;
mod editmode;
mod entity;
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

use allocate::{STACK_ALLOCATOR, SYSTEM_ALLOCATOR, SMemVec};
use entity::{SEntityBucket, SEntityHandle};
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

#[derive(PartialEq, Clone)]
struct SEditModeTranslationDragging {
    entity: SEntityHandle,
    axis: usize,
    start_pos: Vec3,
    mouse_offset: [i32; 2],
}

#[derive(PartialEq, Clone)]
struct SEditModeRotationDragging {
    entity: SEntityHandle,
    axis: usize,
    start_ori: glm::Quat,
    start_entity_to_cursor : Vec3,
}

struct SEditModeContext {
    editing_entity: Option<SEntityHandle>,
    translation_widgets: [model::SModel; 3],
    translation_widget_transforms: [STransform; 3],
    rotation_widgets: [model::SModel; 3],
    rotation_widget_transforms: [STransform; 3],
}

#[derive(PartialEq, Clone)]
enum EEditMode {
    None,
    Translation,
    TranslationDragging(SEditModeTranslationDragging), // axis of translation
    Rotation,
    RotationDragging(SEditModeRotationDragging), // axis of rotation
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

impl SEditModeContext {
    pub fn new(render: &mut render::SRender) -> Result<Self, &'static str> {
        // -- set up translation widget
        let mut translation_widgets = [
            render.new_model("assets/arrow_widget.obj", 1.0, false)?,
            render.new_model("assets/arrow_widget.obj", 1.0, false)?,
            render.new_model("assets/arrow_widget.obj", 1.0, false)?,
        ];
        translation_widgets[0].diffuse_colour = Vec4::new(1.0, 0.0, 0.0, 1.0);
        translation_widgets[1].diffuse_colour = Vec4::new(0.0, 1.0, 0.0, 1.0);
        translation_widgets[2].diffuse_colour = Vec4::new(0.0, 0.0, 1.0, 1.0);

        let mut translation_widget_transforms = [
            STransform::default(),
            STransform::default(),
            STransform::default(),
        ];
        translation_widget_transforms[0].r = glm::quat_angle_axis(utils::PI / 2.0, &Vec3::new(0.0, 1.0, 0.0));
        translation_widget_transforms[1].r = glm::quat_angle_axis(-utils::PI / 2.0, &Vec3::new(1.0, 0.0, 0.0));

        // -- set up rotation widget
        let mut rotation_widgets = [
            render.new_model("assets/ring_widget.obj", 1.0, false)?,
            render.new_model("assets/ring_widget.obj", 1.0, false)?,
            render.new_model("assets/ring_widget.obj", 1.0, false)?,
        ];
        rotation_widgets[0].diffuse_colour = Vec4::new(1.0, 0.0, 0.0, 1.0);
        rotation_widgets[1].diffuse_colour = Vec4::new(0.0, 1.0, 0.0, 1.0);
        rotation_widgets[2].diffuse_colour = Vec4::new(0.0, 0.0, 1.0, 1.0);

        let mut rotation_widget_transforms = [
            STransform::default(),
            STransform::default(),
            STransform::default(),
        ];
        rotation_widget_transforms[0].r = glm::quat_angle_axis(utils::PI / 2.0, &Vec3::new(0.0, 0.0, 1.0));
        rotation_widget_transforms[2].r = glm::quat_angle_axis(utils::PI / 2.0, &Vec3::new(1.0, 0.0, 0.0));

        Ok(Self {
            editing_entity: None,
            translation_widgets,
            translation_widget_transforms,
            rotation_widgets,
            rotation_widget_transforms,
        })
    }
}

impl EEditMode {
    pub fn eats_mouse(&self) -> bool {
        match self {
            Self::TranslationDragging(_) => true,
            Self::RotationDragging(_) => true,
            _ => false,
        }
    }

    pub fn show_translation_widget(&self, query_axis: usize) -> bool {
        match self {
            Self::Translation => true,
            Self::TranslationDragging(data) => data.axis == query_axis,
            _ => false,
        }
    }

    pub fn show_rotation_widget(&self, query_axis: usize) -> bool {
        match self {
            Self::Rotation => true,
            Self::RotationDragging(data) => data.axis == query_axis,
            _ => false,
        }
    }

    pub fn update_translation(
        em: &mut SEditModeContext,
        editmode_input: &editmode::SEditModeInput,
        render: &render::SRender,
        entities: &SEntityBucket
    ) -> EEditMode {
        let e = em.editing_entity.expect("shouldn't be able to translate without entity picked.");

        let cursor_ray = editmode::cursor_ray_world(&editmode_input);
        for axis in 0..=2 {
            if let Some(_) = render.ray_intersects(&em.translation_widgets[axis], &cursor_ray.origin, &cursor_ray.dir, &em.translation_widget_transforms[axis]) {
                let e_pos = entities.get_entity_location(e).t;
                let e_pos_screen = editmode::world_pos_to_screen_pos(&e_pos, &editmode_input);
                let mouse_offset = [(e_pos_screen.x as i32) - editmode_input.mouse_window_pos[0], (e_pos_screen.y as i32) - editmode_input.mouse_window_pos[1]];

                return EEditMode::TranslationDragging(SEditModeTranslationDragging::new(e, axis, e_pos, mouse_offset));
            }
        }


        EEditMode::Translation
    }

    pub fn update_rotation(
        em: &mut SEditModeContext,
        editmode_input: &editmode::SEditModeInput,
        render: &render::SRender,
        entities: &SEntityBucket
    ) -> EEditMode {
        let mut result = EEditMode::Rotation;

        let e = em.editing_entity.expect("shouldn't be able to rotate without entity picked.");
        let cursor_ray = editmode::cursor_ray_world(&editmode_input);
        let mut min_t = None;
        for axis in 0..=2 {
            if let Some(_) = render.ray_intersects(&em.rotation_widgets[axis], &cursor_ray.origin, &cursor_ray.dir, &em.rotation_widget_transforms[axis]) {

                let e_loc = entities.get_entity_location(e);

                let mut plane_normal : Vec3 = glm::zero();
                plane_normal[axis] = 1.0;
                let plane = utils::SPlane::new(&e_loc.t, &plane_normal);
                let cursor_ray_world = editmode::cursor_ray_world(&editmode_input);

                if let Some((cursor_pos_world, t)) = utils::ray_plane_intersection(&cursor_ray_world, &plane) {
                    if min_t.is_none() || min_t.unwrap() > t {
                        let rotation_start_entity_to_cursor = cursor_pos_world - e_loc.t;
                        result = EEditMode::RotationDragging(SEditModeRotationDragging::new(e, axis, e_loc.r, rotation_start_entity_to_cursor));
                        min_t = Some(t);
                    }
                }
            }
        }

        result
    }
}

impl SEditModeTranslationDragging {
    pub fn new(entity: SEntityHandle, axis: usize, start_pos: Vec3, mouse_offset: [i32; 2]) -> Self {
        Self{
            entity,
            axis,
            start_pos,
            mouse_offset,
        }
    }

    pub fn update(
        &self,
        input: &input::SInput,
        editmode_input: &editmode::SEditModeInput,
        render: &mut render::SRender,
        entities: &mut SEntityBucket,
        data_bucket: &databucket::SDataBucket
    ) -> EEditMode {
        if !input.left_mouse_down {
            return EEditMode::Translation;
        }
        else {
            let mut line_dir : Vec3 = glm::zero();
            line_dir[self.axis] = 1.0;

            let line_p0 = self.start_pos + -line_dir;
            let line_p1 = self.start_pos + line_dir;

            let mut render_color : Vec4 = glm::zero();
            render_color[self.axis] = 1.0;
            render_color.w = 1.0;
            render.temp().draw_line(
                &(self.start_pos + -100.0 * line_dir),
                &(self.start_pos + 100.0 * line_dir),
                &render_color,
                true,
                None,
            );

            let offset_mouse_pos = [editmode_input.mouse_window_pos[0] + self.mouse_offset[0],
                                    editmode_input.mouse_window_pos[1] + self.mouse_offset[1]];

            let new_world_pos = editmode::pos_on_screen_space_line_to_world(
                &line_p0,
                &line_p1,
                offset_mouse_pos,
                &editmode_input,
            );

            let mut new_e_loc = entities.get_entity_location(self.entity);
            new_e_loc.t = new_world_pos;

            entities.set_entity_location(
                self.entity,
                new_e_loc,
                &data_bucket,
            );
        }

        return EEditMode::TranslationDragging(self.clone());
    }
}

impl SEditModeRotationDragging {
    pub fn new(entity: SEntityHandle, axis: usize, start_ori: glm::Quat, start_entity_to_cursor: Vec3) -> Self {
        Self{
            entity,
            axis,
            start_ori,
            start_entity_to_cursor,
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
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();
    let mut window = render.create_window(&windowclass, "rusgam", 1600, 900)?;

    window.init_render_target_views(render.device())?;
    window.show();

    let mut editmode_ctxt = SEditModeContext::new(&mut render).unwrap();


    let mut data_bucket = databucket::SDataBucket::new(256, &SYSTEM_ALLOCATOR);

    let entities = SEntityBucket::new(67485, 16);
    data_bucket.add_entities(entities);
    data_bucket.add_renderer(render);
    let bvh = bvh::STree::new();
    data_bucket.add_bvh(bvh);

    let rotating_entity = entitytypes::testtexturedcubeentity::create(
        &data_bucket, Some("tst_rotating"),
        STransform::new_translation(&glm::Vec3::new(0.0, 0.0, 0.0)))?;
    entitytypes::testtexturedcubeentity::create(
        &data_bucket, Some("tst_textured_cube"),
        STransform::new_translation(&glm::Vec3::new(3.0, 0.0, 0.0)))?;
    entitytypes::flatshadedcubeentity::create(
        &data_bucket, Some("tst_coloured_cube"), Some(glm::Vec4::new(1.0, 0.0, 0.0, 0.9)),
        STransform::new_translation(&glm::Vec3::new(0.0, 2.0, 0.0)))?;
    entitytypes::testopenroomentity::create(
        &data_bucket, Some("tst_room"),
        STransform::new_translation(&glm::Vec3::new(0.0, -2.0, 0.0)))?;

    // -- update loop

    let mut _framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let start_time = winapi.curtimemicroseconds();
    let _rot_axis = Vec3::new(0.0, 1.0, 0.0);

    let mut camera = camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0));

    let mut input = input::SInput::new();

    let mut mode = EMode::Edit;
    let mut edit_mode = EEditMode::Translation;

    let mut draw_selected_bvh  = false;

    let mut show_imgui_demo_window = false;

    let mut gjk_debug = gjk::SGJKDebug::new(&data_bucket);

    while !input.q_down {
        // -- handle edit mode toggles
        if input.tilde_edge.down() {
            mode.toggle(&mut edit_mode);
        }

        if mode == EMode::Edit {
            if input.t_edge.down() {
                edit_mode = EEditMode::Translation;
            }
            else if input.r_edge.down() {
                edit_mode = EEditMode::Rotation;
            }
        }

        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;
        let dts = (dt as f32) / 1_000_000.0;

        let _total_time = curframetime - start_time;

        // -- update
        let cur_angle = ((_total_time as f32) / 1_000_000.0) * (3.14159 / 4.0);
        data_bucket.get_entities().unwrap().with_mut(|entities: &mut SEntityBucket| {
            entities.set_entity_location(rotating_entity, STransform::new_rotation(&glm::quat_angle_axis(cur_angle, &_rot_axis)), &data_bucket);
        });

        //let mut fixed_size_model_xform = STransform::new_translation(&glm::Vec3::new(0.0, 5.0, 0.0));

        let mut can_rotate_camera = false;
        if let EMode::Play = mode {
            can_rotate_camera = true;
        }
        else if input.middle_mouse_down {
            can_rotate_camera = true;
        }
        camera.update_from_input(&input, dts, can_rotate_camera);

        let editmode_input = data_bucket.get_renderer().unwrap().with(|render: &render::SRender| {
            editmode::SEditModeInput::new_for_frame(&window, &winapi, &camera, &render)
        });

        input.mouse_dx = 0;
        input.mouse_dy = 0;
        let view_matrix = camera.world_to_view_matrix();
        let cursor_ray = editmode::cursor_ray_world(&editmode_input);

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);

        // -- check if the user clicked an edit widget
        if input.left_mouse_edge.down() && !imgui_ctxt.io().want_capture_mouse && mode == EMode::Edit && editmode_ctxt.editing_entity.is_some() {
            data_bucket.get_renderer().unwrap().with(|render: &render::SRender| {
                data_bucket.get_entities().unwrap().with(|entities: &SEntityBucket| {
                    if edit_mode == EEditMode::Translation {
                        edit_mode = EEditMode::update_translation(&mut editmode_ctxt, &editmode_input, &render, &entities);
                    }
                    else if edit_mode == EEditMode::Rotation {
                        edit_mode = EEditMode::update_rotation(&mut editmode_ctxt, &editmode_input, &render, &entities);
                    }
                });
            });
        }

        // -- handle translation/rotation edit mode mouse input
        if mode == EMode::Edit {
            data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
                data_bucket.get_entities().unwrap().with_mut(|entities: &mut SEntityBucket| {
                    if let EEditMode::TranslationDragging(data) = edit_mode.clone() {
                        edit_mode = data.update(&input, &editmode_input, render, entities, &data_bucket);
                    }
                    else if let EEditMode::RotationDragging(data) = edit_mode.clone() {
                        if !input.left_mouse_down {
                            edit_mode = EEditMode::Rotation;
                        }
                        else {
                            assert!(editmode_ctxt.editing_entity.is_some(), "rotating but no entity!");

                            let e_loc = entities.get_entity_location(editmode_ctxt.editing_entity.expect(""));

                            let mut plane_normal : Vec3 = glm::zero();
                            plane_normal[data.axis] = 1.0;
                            let plane = utils::SPlane::new(&e_loc.t, &plane_normal);

                            let cursor_ray_world = editmode::cursor_ray_world(&editmode_input);
                            if let Some((cursor_pos_world, _)) = utils::ray_plane_intersection(&cursor_ray_world, &plane) {
                                let entity_to_cursor = cursor_pos_world - e_loc.t;

                                let rotation = glm::quat_rotation(&data.start_entity_to_cursor,
                                                                  &entity_to_cursor);

                                let new_entity_ori = rotation * data.start_ori;

                                let mut new_e_loc = e_loc;
                                new_e_loc.r = new_entity_ori;

                                entities.set_entity_location(
                                    editmode_ctxt.editing_entity.expect(""),
                                    new_e_loc,
                                    &data_bucket,
                                );

                                let mut render_color : Vec4 = glm::zero();
                                render_color[data.axis] = 1.0;
                                render_color.w = 1.0;
                                render.temp().draw_line(
                                    &e_loc.t,
                                    &(e_loc.t + data.start_entity_to_cursor),
                                    &render_color,
                                    true,
                                    None,
                                );
                                render.temp().draw_line(
                                    &e_loc.t,
                                    &cursor_pos_world,
                                    &render_color,
                                    true,
                                    None,
                                );
                            }
                        }
                    }
                });
            });
        }

        // -- update edit widgets
        if mode == EMode::Edit {
            if let Some(e) = editmode_ctxt.editing_entity {
                data_bucket.get_entities().unwrap().with(|entities: &SEntityBucket| {
                    editmode_ctxt.translation_widget_transforms[0].t = entities.get_entity_location(e).t;
                    editmode_ctxt.translation_widget_transforms[1].t = entities.get_entity_location(e).t;
                    editmode_ctxt.translation_widget_transforms[2].t = entities.get_entity_location(e).t;
                    //println!("Set translation widget: {:?}", translation_widget_transform.t);
                    editmode::scale_to_fixed_screen_size(&mut editmode_ctxt.translation_widget_transforms[0], 0.02, &editmode_input);
                    editmode::scale_to_fixed_screen_size(&mut editmode_ctxt.translation_widget_transforms[1], 0.02, &editmode_input);
                    editmode::scale_to_fixed_screen_size(&mut editmode_ctxt.translation_widget_transforms[2], 0.02, &editmode_input);

                    editmode_ctxt.rotation_widget_transforms[0].t = entities.get_entity_location(e).t;
                    editmode_ctxt.rotation_widget_transforms[1].t = entities.get_entity_location(e).t;
                    editmode_ctxt.rotation_widget_transforms[2].t = entities.get_entity_location(e).t;
                    //println!("Set translation widget: {:?}", translation_widget_transform.t);
                    editmode::scale_to_fixed_screen_size(&mut editmode_ctxt.rotation_widget_transforms[0], 0.034, &editmode_input);
                    editmode::scale_to_fixed_screen_size(&mut editmode_ctxt.rotation_widget_transforms[1], 0.034, &editmode_input);
                    editmode::scale_to_fixed_screen_size(&mut editmode_ctxt.rotation_widget_transforms[2], 0.034, &editmode_input);
                });
            }
        }

        // -- draw edit widgets
        if mode == EMode::Edit && editmode_ctxt.editing_entity.is_some() {
            data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
                for axis in 0..=2 {
                    if edit_mode.show_translation_widget(axis) {
                            render.temp().draw_model(&editmode_ctxt.translation_widgets[axis], &editmode_ctxt.translation_widget_transforms[axis], true);
                    }
                }
                for axis in 0..=2 {
                    if edit_mode.show_rotation_widget(axis) {
                        render.temp().draw_model(&editmode_ctxt.rotation_widgets[axis], &editmode_ctxt.rotation_widget_transforms[axis], true);
                    }
                }
            });
        }

        // -- update IMGUI
        let io = imgui_ctxt.io_mut();
        io.display_size = [window.width() as f32, window.height() as f32];
        let imgui_want_capture_mouse = io.want_capture_mouse;

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

                data_bucket.get_bvh().unwrap().with(|bvh: &bvh::STree| {
                    bvh.imgui_menu(&imgui_ui, &mut draw_selected_bvh);
                });

                gjk_debug.imgui_menu(&imgui_ui, &data_bucket, editmode_ctxt.editing_entity, Some(rotating_entity));

            });

            if let Some(e) = editmode_ctxt.editing_entity {
                data_bucket.get_entities().unwrap().with_mut(|entities: &mut SEntityBucket| {
                    entities.show_imgui_window(e, &imgui_ui);
                });
            }
        }

        // -- draw selected object's BVH heirarchy
        if draw_selected_bvh {
            if let Some(e) = editmode_ctxt.editing_entity {
                STACK_ALLOCATOR.with(|sa| {
                    data_bucket.get_entities().unwrap().with(|entities: &SEntityBucket| {
                        data_bucket.get_bvh().unwrap().with(|bvh: &bvh::STree| {
                            data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
                                let mut aabbs = SMemVec::new(sa, 32, 0).unwrap();
                                bvh.get_bvh_heirarchy_for_entry(entities.get_entity_bvh_entry(e), &mut aabbs);
                                for aabb in aabbs.as_slice() {
                                    render.temp().draw_aabb(aabb, &Vec4::new(1.0, 0.0, 0.0, 1.0), true);
                                }
                            });
                        });
                    });
                });
            }
        }

        // -- draw selected object colliding/not with rotating_entity
        if let Some(e) = editmode_ctxt.editing_entity {
            STACK_ALLOCATOR.with(|sa| {
                data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
                    data_bucket.get_entities().unwrap().with(|entities: &SEntityBucket| {
                        let loc = entities.get_entity_location(e);

                        let world_verts = {
                            let model = entities.get_entity_model(e).unwrap();
                            let per_vert_data = render.mesh_loader().get_per_vertex_data(model.mesh);

                            let mut world_verts = SMemVec::new(sa, per_vert_data.len(), 0).unwrap();

                            for vd in per_vert_data.as_slice() {
                                world_verts.push(loc.mul_point(&vd.position));
                            }

                            world_verts
                        };

                        let rot_box_world_verts = {
                            let model = entities.get_entity_model(rotating_entity).unwrap();
                            let loc = entities.get_entity_location(rotating_entity);
                            let per_vert_data = render.mesh_loader().get_per_vertex_data(model.mesh);

                            let mut world_verts = SMemVec::new(sa, per_vert_data.len(), 0).unwrap();

                            for vd in per_vert_data.as_slice() {
                                world_verts.push(loc.mul_point(&vd.position));
                            }

                            world_verts
                        };

                        if gjk::gjk(world_verts.as_slice(), rot_box_world_verts.as_slice()) {
                            render.temp().draw_sphere(&loc.t, 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.1), true, None);
                        }
                    });
                });
            });
        }

        STACK_ALLOCATOR.with(|sa| -> Result<(), &'static str> {
            let imgui_draw_data = imgui_ui.render();
            data_bucket.get_entities().unwrap().with(|entities: &SEntityBucket| {
                data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
                    let (_entities, model_xforms, models) = entities.build_render_data(sa);

                    // -- render world
                    let render_result = render.render_frame(&mut window, &view_matrix, models.as_slice(), model_xforms.as_slice(), Some(&imgui_draw_data));
                    match render_result {
                        Ok(_) => {},
                        Err(e) => {
                            println!("ERROR: render failed with error '{}'", e);
                            panic!();
                        },
                    }
                });
            });

            Ok(())
        })?;

        // -- cast rays against world
        if input.left_mouse_edge.down() && !imgui_want_capture_mouse && !edit_mode.eats_mouse() {
            data_bucket.get_bvh().unwrap().with(|bvh: &bvh::STree| {
                let entity_hit = bvh.cast_ray(&data_bucket, &cursor_ray);
                if entity_hit.is_some() {
                    editmode_ctxt.editing_entity = entity_hit;
                }
            });
        }

        lastframetime = curframetime;
        _framecount += 1;


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

                        data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
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
    data_bucket.get_renderer().unwrap().with_mut(|render: &mut render::SRender| {
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

    main_d3d12().unwrap();
}
