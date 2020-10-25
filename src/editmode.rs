use camera;
use glm::{Vec3, Vec4};
use niced3d12 as n12;
use render;
use rustywindows;
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
}

impl SEditModeInput {
    pub fn new_for_frame(
        window: &n12::SD3D12Window,
        winapi: &rustywindows::SWinAPI,
        camera: &camera::SCamera,
        render: &render::SRender,
    ) -> Self {
        Self {
            window_width: window.width(),
            window_height: window.height(),
            mouse_window_pos: window.mouse_pos(&winapi.rawwinapi()),
            camera_pos_world: camera.pos_world,
            camera_forward: camera.forward_world(),
            world_to_view_matrix: camera.world_to_view_matrix(),
            fovy: render.fovy(),
            znear: render.znear(),
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


