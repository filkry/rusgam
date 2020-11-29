use glm::{Vec3, Mat4};

use game_context::{SGameContext, SFrameContext};
use game_mode;
use input;

pub struct SDebugFPCamera {
    pub pos_world: Vec3,
    x_angle: f32,
    y_angle: f32,
}

impl SDebugFPCamera {
    const MAX_X_DELTA : f32 = std::f32::consts::PI / 2.5;
    const TWOPI : f32 = std::f32::consts::PI * 2.0;

    fn forward_local() -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn right_local() -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }

    fn up_world() -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }

    pub fn forward_world(&self) -> Vec3 {
        let rotate_x = glm::quat_angle_axis(self.x_angle, &Self::right_local());
        let rotate_y = glm::quat_angle_axis(self.y_angle, &Self::up_world());

        let rotate = rotate_y * rotate_x;

        let forward_world = glm::quat_rotate_vec3(&rotate, &Self::forward_local());
        return forward_world;
    }

    pub fn new(pos: Vec3) -> Self {
        Self {
            pos_world: pos,
            x_angle: 0.0,
            y_angle: 0.0,
        }
    }

    pub fn update_from_input(&mut self, input: &input::SInput, dts: f32, can_rotate_camera: bool) {
        let forward_world = glm::rotate_y_vec3(&Self::forward_local(), self.y_angle);
        let right_world = glm::rotate_y_vec3(&Self::right_local(), self.y_angle);

        const SPEED: f32 = 5.0;

        if input.w_down {
            self.pos_world = self.pos_world + forward_world * SPEED * dts;
        }
        if input.s_down {
            self.pos_world = self.pos_world + forward_world * -SPEED * dts;
        }
        if input.a_down {
            self.pos_world = self.pos_world + right_world * -SPEED * dts;
        }
        if input.d_down {
            self.pos_world = self.pos_world + right_world * SPEED * dts;
        }
        if input.space_down {
            self.pos_world = self.pos_world + Self::up_world() * SPEED * dts;
        }
        if input.c_down {
            self.pos_world = self.pos_world + Self::up_world() * -SPEED * dts;
        }

        if can_rotate_camera {
            if input.mouse_dy != 0 {
                self.x_angle = super::utils::clamp(
                    self.x_angle + ((input.mouse_dy as f32) / 100.0),
                    -Self::MAX_X_DELTA,
                    Self::MAX_X_DELTA
                );
            }

            if input.mouse_dx != 0 {
                self.y_angle = (self.y_angle + ((input.mouse_dx as f32) / 100.0)) % Self::TWOPI;
            }
        }
    }

    pub fn world_to_view_matrix(&self) -> Mat4 {
        glm::look_at_lh(&self.pos_world, &(self.pos_world + self.forward_world()), &Self::up_world())
    }
}

pub fn update_debug_camera(game_context: &SGameContext, frame_context: &SFrameContext) {
    let mut can_rotate_camera = false;
    game_context.data_bucket.get::<SDebugFPCamera>()
        .and::<game_mode::SGameMode>()
        .and::<input::SInput>()
        .with_mcc(|camera, game_mode, input| {
            if let game_mode::EMode::Play = game_mode.mode {
                can_rotate_camera = true;
            }
            else if input.middle_mouse_down {
                can_rotate_camera = true;
            }
            camera.update_from_input(&input, frame_context.dt_s, can_rotate_camera);
        });
}