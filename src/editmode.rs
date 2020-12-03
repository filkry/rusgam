use allocate::{STACK_ALLOCATOR};
use bvh;
use camera;
use collections::{SVec};
use databucket;
use game_context::{SGameContext, SFrameContext};
use game_mode;
use entity::{SEntityBucket, SEntityHandle};
use glm::{Vec3, Vec4};
use input;
use model;
use render;
use utils;
use utils::{STransform};

pub struct SEditModeInput {
    pub window_width: u32,
    pub window_height: u32,
    pub mouse_window_pos: [i32; 2],
    pub camera_pos_world: Vec3,
    pub camera_forward: Vec3,
    pub world_to_view_matrix: glm::Mat4,
    pub fovy: f32,
    pub znear: f32,

    pub imgui_want_capture_mouse: bool,
}

#[derive(PartialEq, Clone)]
pub struct SEditModeTranslationDragging {
    entity: SEntityHandle,
    axis: usize,
    start_pos: Vec3,
    mouse_offset: [i32; 2],
}

#[derive(PartialEq, Clone)]
pub struct SEditModeRotationDragging {
    entity: SEntityHandle,
    axis: usize,
    start_ori: glm::Quat,
    start_entity_to_cursor : Vec3,
}

pub struct SEditModeContext {
    editing_entity: Option<SEntityHandle>,
    translation_widgets: [model::SModel; 3],
    translation_widget_transforms: [STransform; 3],
    rotation_widgets: [model::SModel; 3],
    rotation_widget_transforms: [STransform; 3],

    clicked_entity: Option<SEntityHandle>,
    can_select_clicked_entity: bool,
}

#[derive(PartialEq, Clone)]
pub enum EEditMode {
    None,
    Translation,
    TranslationDragging(SEditModeTranslationDragging), // axis of translation
    Rotation,
    RotationDragging(SEditModeRotationDragging), // axis of rotation
}

impl SEditModeInput {
    pub fn new_for_frame(
        window_width: u32,
        window_height: u32,
        camera: &camera::SDebugFPCamera,
        input: &input::SInput,
        render: &render::SRender,
        imgui_want_capture_mouse: bool,
    ) -> Self {
        Self {
            window_width,
            window_height,
            mouse_window_pos: input.mouse_cursor_pos_window,
            camera_pos_world: camera.pos_world,
            camera_forward: camera.forward_world(),
            world_to_view_matrix: camera.world_to_view_matrix(),
            fovy: render.fovy(),
            znear: render.znear(),
            imgui_want_capture_mouse,
        }
    }
}

impl SEditModeContext {
    pub fn new(render: &mut render::SRender) -> Result<Self, &'static str> {
        // -- set up translation widget
        let mut translation_widgets = [
            render.new_model_from_obj("assets/arrow_widget.obj", 1.0, false)?,
            render.new_model_from_obj("assets/arrow_widget.obj", 1.0, false)?,
            render.new_model_from_obj("assets/arrow_widget.obj", 1.0, false)?,
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
            render.new_model_from_obj("assets/ring_widget.obj", 1.0, false)?,
            render.new_model_from_obj("assets/ring_widget.obj", 1.0, false)?,
            render.new_model_from_obj("assets/ring_widget.obj", 1.0, false)?,
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

            clicked_entity: None,
            can_select_clicked_entity: false,
        })
    }

    pub fn editing_entity(&self) -> Option<SEntityHandle> {
        self.editing_entity
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
        editmode_input: &SEditModeInput,
        input: &input::SInput,
        render: &render::SRender,
        entities: &SEntityBucket,
    ) -> EEditMode {
        if editmode_input.imgui_want_capture_mouse || !input.left_mouse_edge.down() {
            return EEditMode::Translation;
        }

        let e = em.editing_entity.expect("shouldn't be able to translate without entity picked.");

        let cursor_ray = cursor_ray_world(&editmode_input);
        for axis in 0..=2 {
            if let Some(_) = render.ray_intersects(&em.translation_widgets[axis], &cursor_ray.origin, &cursor_ray.dir, &em.translation_widget_transforms[axis]) {
                let e_pos = entities.get_entity_location(e).t;
                let e_pos_screen = world_pos_to_screen_pos(&e_pos, &editmode_input);
                let mouse_offset = [(e_pos_screen.x as i32) - editmode_input.mouse_window_pos[0], (e_pos_screen.y as i32) - editmode_input.mouse_window_pos[1]];

                em.can_select_clicked_entity = false;

                return EEditMode::TranslationDragging(SEditModeTranslationDragging::new(e, axis, e_pos, mouse_offset));
            }
        }


        EEditMode::Translation
    }

    pub fn update_rotation(
        em: &mut SEditModeContext,
        editmode_input: &SEditModeInput,
        input: &input::SInput,
        render: &render::SRender,
        entities: &SEntityBucket
    ) -> EEditMode {
        let mut result = EEditMode::Rotation;

        if editmode_input.imgui_want_capture_mouse || !input.left_mouse_edge.down() {
            return result;
        }

        let e = em.editing_entity.expect("shouldn't be able to rotate without entity picked.");
        let cursor_ray = cursor_ray_world(&editmode_input);
        let mut min_t = None;
        for axis in 0..=2 {
            if let Some(_) = render.ray_intersects(&em.rotation_widgets[axis], &cursor_ray.origin, &cursor_ray.dir, &em.rotation_widget_transforms[axis]) {

                let e_loc = entities.get_entity_location(e);

                let mut plane_normal : Vec3 = glm::zero();
                plane_normal[axis] = 1.0;
                let plane = utils::SPlane::new(&e_loc.t, &plane_normal);
                let cursor_ray_world = cursor_ray_world(&editmode_input);

                em.can_select_clicked_entity = false;

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

    pub fn update(
        &self,
        gc: &SGameContext,
        ctxt: &mut SEditModeContext,
        em_input: &SEditModeInput,
        input: &input::SInput,
        data_bucket: &databucket::SDataBucket
    ) -> Self {
        let mut mode = self.clone();

        drop(self);

        let cursor_ray = cursor_ray_world(&em_input);

        // -- cast ray to select entity for edit mode
        ctxt.clicked_entity = None;
        if input.left_mouse_edge.down() && !em_input.imgui_want_capture_mouse && !mode.eats_mouse() {
            data_bucket.get::<bvh::STree<SEntityHandle>>().with(|bvh: &bvh::STree<SEntityHandle>| {

                STACK_ALLOCATOR.with(|sa| {
                    let mut bvh_results = SVec::<(f32, SEntityHandle)>::new(&sa.as_ref(), 256, 0).unwrap();
                    bvh.cast_ray(&cursor_ray, &mut bvh_results);

                    let mut min_t : Option::<f32> = None;
                    let mut min_entity : Option<SEntityHandle> = None;

                    for (t, entity) in bvh_results.as_ref() {
                        if *t < min_t.unwrap_or(std::f32::MAX) {
                            if let Some(t_mesh) = render::cast_ray_against_entity_model(data_bucket, &cursor_ray, *entity) {
                                min_t = Some(t_mesh);
                                min_entity = Some(*entity);
                            }
                        }
                    }

                    if let Some(_) = min_entity {
                        ctxt.clicked_entity = min_entity;
                        ctxt.can_select_clicked_entity = true;
                    }
                });

            });
        }

        // -- toggle edit modes
        if input.t_edge.down() && ctxt.editing_entity.is_some() {
            mode = EEditMode::Translation;
        }
        else if input.r_edge.down() && ctxt.editing_entity.is_some() {
            mode = EEditMode::Rotation;
        }

        data_bucket.get_renderer()
            .and::<SEntityBucket>()
            .with_mm(|render, entities| {
                if mode == EEditMode::Translation {
                    mode = EEditMode::update_translation(ctxt, &em_input, &input, &render, &entities);
                }
                else if mode == EEditMode::Rotation {
                    mode = EEditMode::update_rotation(ctxt, &em_input, &input, &render, &entities);
                }
                else if let EEditMode::TranslationDragging(data) = mode.clone() {
                    mode = data.update(&input, &em_input, gc, render, entities);
                }
                else if let EEditMode::RotationDragging(data) = mode.clone() {
                    mode = data.update(&input, &em_input, gc, render, entities);
                }
            });

        if ctxt.can_select_clicked_entity && ctxt.clicked_entity.is_some() {
            ctxt.editing_entity = ctxt.clicked_entity;
        }

        // -- move/scale edit widgets
        if let Some(e) = ctxt.editing_entity {
            data_bucket.get_entities().with(|entities: &SEntityBucket| {
                ctxt.translation_widget_transforms[0].t = entities.get_entity_location(e).t;
                ctxt.translation_widget_transforms[1].t = entities.get_entity_location(e).t;
                ctxt.translation_widget_transforms[2].t = entities.get_entity_location(e).t;
                //println!("Set translation widget: {:?}", translation_widget_transform.t);
                scale_to_fixed_screen_size(&mut ctxt.translation_widget_transforms[0], 0.02, &em_input);
                scale_to_fixed_screen_size(&mut ctxt.translation_widget_transforms[1], 0.02, &em_input);
                scale_to_fixed_screen_size(&mut ctxt.translation_widget_transforms[2], 0.02, &em_input);

                ctxt.rotation_widget_transforms[0].t = entities.get_entity_location(e).t;
                ctxt.rotation_widget_transforms[1].t = entities.get_entity_location(e).t;
                ctxt.rotation_widget_transforms[2].t = entities.get_entity_location(e).t;
                //println!("Set translation widget: {:?}", translation_widget_transform.t);
                scale_to_fixed_screen_size(&mut ctxt.rotation_widget_transforms[0], 0.034, &em_input);
                scale_to_fixed_screen_size(&mut ctxt.rotation_widget_transforms[1], 0.034, &em_input);
                scale_to_fixed_screen_size(&mut ctxt.rotation_widget_transforms[2], 0.034, &em_input);
            });

            // -- draw edit widgets
            data_bucket.get_renderer().with_mut(|render: &mut render::SRender| {
                for axis in 0..=2 {
                    if mode.show_translation_widget(axis) {
                            render.temp().draw_model(&ctxt.translation_widgets[axis], &ctxt.translation_widget_transforms[axis], true);
                    }
                }
                for axis in 0..=2 {
                    if mode.show_rotation_widget(axis) {
                        render.temp().draw_model(&ctxt.rotation_widgets[axis], &ctxt.rotation_widget_transforms[axis], true);
                    }
                }
            });
        }

        return mode;
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
        editmode_input: &SEditModeInput,
        gc: &super::SGameContext,
        render: &mut render::SRender,
        entities: &mut SEntityBucket,
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

            let new_world_pos = pos_on_screen_space_line_to_world(
                &line_p0,
                &line_p1,
                offset_mouse_pos,
                &editmode_input,
            );

            let mut new_e_loc = entities.get_entity_location(self.entity);
            new_e_loc.t = new_world_pos;

            entities.set_location(gc, self.entity, new_e_loc);
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

    pub fn update(
        &self,
        input: &input::SInput,
        editmode_input: &SEditModeInput,
        gc: &super::SGameContext,
        render: &mut render::SRender,
        entities: &mut SEntityBucket,
    ) -> EEditMode {
        if !input.left_mouse_down {
            return EEditMode::Rotation;
        }
        else {
            let e_loc = entities.get_entity_location(self.entity);

            let mut plane_normal : Vec3 = glm::zero();
            plane_normal[self.axis] = 1.0;
            let plane = utils::SPlane::new(&e_loc.t, &plane_normal);

            let cursor_ray_world = cursor_ray_world(&editmode_input);
            if let Some((cursor_pos_world, _)) = utils::ray_plane_intersection(&cursor_ray_world, &plane) {
                let entity_to_cursor = cursor_pos_world - e_loc.t;

                let rotation = glm::quat_rotation(&self.start_entity_to_cursor,
                                                  &entity_to_cursor);

                let new_entity_ori = rotation * self.start_ori;

                let mut new_e_loc = e_loc;
                new_e_loc.r = new_entity_ori;

                entities.set_location(gc, self.entity, new_e_loc);

                let mut render_color : Vec4 = glm::zero();
                render_color[self.axis] = 1.0;
                render_color.w = 1.0;
                render.temp().draw_line(
                    &e_loc.t,
                    &(e_loc.t + self.start_entity_to_cursor),
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

            return EEditMode::RotationDragging(self.clone());
        }
    }
}


// -- transform, camera pos, and camera forward must be in the same space
pub fn scale_to_fixed_screen_size(
    transform: &mut STransform,
    pct_of_near_plane_for_one_unit: f32,
    editmode_input: &SEditModeInput,
) {
    let fovx = utils::fovx(editmode_input.fovy, editmode_input.window_width, editmode_input.window_height);

    let to_fixed = transform.t - editmode_input.camera_pos_world;
    let dist = glm::length(&to_fixed);

    let angle_from_forward = glm::angle(&to_fixed, &editmode_input.camera_forward);
    let proj_dist = editmode_input.znear / (angle_from_forward).cos();

    // -- the whole idea of this code is to build a ratio of the similar
    // -- triangle from the object in world space to the amount of space
    // -- 1 unit will take up on the near plane projection, then scale it
    // -- so that space is constant
    let proj_ratio = proj_dist / dist;

    let unit_in_proj_space = 1.0 * proj_ratio;

    let total_proj_space = 2.0 * editmode_input.znear * (fovx / 2.0).tan();
    let desired_proj_space = total_proj_space * pct_of_near_plane_for_one_unit;

    let scale = desired_proj_space / unit_in_proj_space;

    transform.s = scale;
}

pub fn cursor_ray_world(
    editmode_input: &SEditModeInput,
) -> utils::SRay {
    let (x_pos, y_pos) = (editmode_input.mouse_window_pos[0], editmode_input.mouse_window_pos[1]);

    //println!("Left button down: {}, {}", x_pos, y_pos);

    let half_camera_near_clip_height = (editmode_input.fovy/2.0).tan() * editmode_input.znear;
    let half_camera_near_clip_width = ((editmode_input.window_width as f32) / (editmode_input.window_height as f32)) * half_camera_near_clip_height;

    let near_clip_top_left_camera_space = Vec3::new(-half_camera_near_clip_width, half_camera_near_clip_height, editmode_input.znear);
    let near_clip_deltax_camera_space = Vec3::new(2.0 * half_camera_near_clip_width, 0.0, 0.0);
    let near_clip_deltay_camera_space = Vec3::new(0.0, -2.0 * half_camera_near_clip_height, 0.0);

    let pct_width = (x_pos as f32) / (editmode_input.window_width as f32);
    let pct_height = (y_pos as f32) / (editmode_input.window_height as f32);

    let to_z_near_camera_space = near_clip_top_left_camera_space +
        pct_width * near_clip_deltax_camera_space +
        pct_height * near_clip_deltay_camera_space;

    //println!("to_z_near_camera_space: {:?}", to_z_near_camera_space);

    let world_to_view = editmode_input.world_to_view_matrix;
    let view_to_world = glm::inverse(&world_to_view);

    let to_z_near_world_space = view_to_world * utils::vec3_to_homogenous(&to_z_near_camera_space, 0.0);

    utils::SRay{
        origin: editmode_input.camera_pos_world,
        dir: to_z_near_world_space.xyz(),
    }
}

pub fn world_pos_to_screen_pos(
    world_pos: &Vec3,
    editmode_input: &SEditModeInput,
) -> Vec3 {
    let perspective_matrix = {
        let aspect = (editmode_input.window_width as f32) / (editmode_input.window_height as f32);
        let zfar = 100.0;

        //SMat44::new_perspective(aspect, fovy, znear, zfar)
        glm::perspective_lh_zo(aspect, editmode_input.fovy, editmode_input.znear, zfar)
    };

    let view_perspective_matrix = perspective_matrix * editmode_input.world_to_view_matrix;

    let pos_clip_space = view_perspective_matrix * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    let pos_ndc = pos_clip_space / pos_clip_space.w;

    let width_f32 = editmode_input.window_width as f32;
    let height_f32 = editmode_input.window_height as f32;

    let screen_space = Vec3::new(
        ((pos_ndc.x + 1.0) / 2.0) * width_f32,
        ((-pos_ndc.y + 1.0) / 2.0) * height_f32,
        0.0, // not valid
    );

    screen_space
}

pub fn pos_on_screen_space_line_to_world(
    world_line_p0: &Vec3,
    world_line_p1: &Vec3,
    screen_space_pos: [i32; 2],
    editmode_input: &SEditModeInput,
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
        let aspect = (editmode_input.window_width as f32) / (editmode_input.window_height as f32);
        let zfar = 100.0;

        //SMat44::new_perspective(aspect, fovy, znear, zfar)
        glm::perspective_lh_zo(aspect, editmode_input.fovy, editmode_input.znear, zfar)
    };

    let view_perspective_matrix = perspective_matrix * editmode_input.world_to_view_matrix;

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

    let width_f32 = editmode_input.window_width as f32;
    let height_f32 = editmode_input.window_height as f32;

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

    //println!("Line p0 screen space: {:?}", line_p0_screen_space);
    //println!("Line p1 screen space: {:?}", line_p1_screen_space);

    // -- mouse is thought of as on the znear plane, so 0.0
    let mouse_pos_v = Vec3::new(screen_space_pos[0] as f32, screen_space_pos[1] as f32, 0.0);

    let (closest_pos_screen_space, _) = utils::closest_point_on_line(&line_p0_screen_space, &line_p1_screen_space, &mouse_pos_v);
    //println!("closest pos screen space: {:?}", closest_pos_screen_space);

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

pub fn update_create_input_for_frame(game_context: &SGameContext, frame_context: &SFrameContext) -> SEditModeInput {
    game_context.data_bucket.get_renderer()
        .and::<camera::SDebugFPCamera>()
        .and::<input::SInput>()
        .with_ccc(|render, camera, input| {
            SEditModeInput::new_for_frame(
                frame_context.window_width,
                frame_context.window_height,
                camera,
                input,
                render,
                frame_context.imgui_want_capture_mouse,
            )
        })
}

pub fn update_edit_mode(game_context: &SGameContext, frame_context: &SFrameContext) {
    game_context.data_bucket.get::<game_mode::SGameMode>()
        .and::<input::SInput>()
        .with_mc(|game_mode, input| {
            frame_context.data_bucket.get::<SEditModeInput>()
                .with(|editmode_input| {
                    if game_mode.mode == game_mode::EMode::Edit {
                        game_mode.edit_mode = game_mode.edit_mode.update(&game_context, &mut game_mode.edit_mode_ctxt, &editmode_input, &input, &game_context.data_bucket);
                    }
                });
        });
}