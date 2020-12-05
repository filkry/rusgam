// -- I still don't want to implement my own types, but I'm tired of how gnarly glm types look
// -- in the debugger. So I'm going to try creating memory-layout-equivalent types, and unsafely
// -- casting to glm types in my operations
use utils;

#[derive(Copy, Clone, PartialEq)]
pub struct Vec2{
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Vec3{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Vec4{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Quat{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub struct Mat4{
    data: [f32; 16],
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3{
            x,
            y,
            z,
        }
    }

    pub fn zero() -> Vec3 {
        Self::new(0, 0, 0)
    }

    pub fn sqmag(&self) -> f32 {
        Self::dot(self, self)
    }

    pub fn mag(&self) -> f32 {
        self.sqmag().sqrt()
    }

    pub fn dot(a: &Self, b: &Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    pub fn cross(a: &Self, b: &Self) -> Vec3 {
        panic!("not implemented");
    }

    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Vec3{
            x: utils::lerp(a.x, b.x, t),
            y: utils::lerp(a.y, b.y, t),
            z: utils::lerp(a.z, b.z, t),
        }
    }

    pub fn min(a: &Self, b: &Self) -> Self {
        Self::new(
            a.x.min(b.x),
            a.y.min(b.y),
            a.z.min(b.z),
        )
    }

    pub fn max(a: &Self, b: &Self) -> Self {
        Self::new(
            a.x.max(b.x),
            a.y.max(b.y),
            a.z.max(b.z),
        )
    }

    pub fn angle_between(a: &Self, b: &Self) -> f32 {
        panic!("Not implemented");
    }

    pub fn rotate_y(&self, angle: f32) -> Self {
        panic!("Not implemented yet");
    }
}

impl std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl std::ops::Mul<&Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl std::ops::Add<Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub<Vec3> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Sub<Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Sub<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Vec3 index out of bounds"),
        }
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self{
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3{
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4{
            x,
            y,
            z,
            w,
        }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn xyz(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn sqmag(&self) -> f32 {
        Self::dot(self, self)
    }

    pub fn mag(&self) -> f32 {
        self.sqmag().sqrt()
    }

    pub fn dot(a: &Self, b: &Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }

    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Self{
            x: utils::lerp(a.x, b.x, t),
            y: utils::lerp(a.y, b.y, t),
            z: utils::lerp(a.z, b.z, t),
            w: utils::lerp(a.w, b.w, t),
        }
    }
}

impl std::ops::Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(self * rhs.x, self * rhs.y, self * rhs.z, self * rhs.w)
    }
}

impl std::ops::Mul<&Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, rhs: &Vec4) -> Self::Output {
        Vec4::new(self * rhs.x, self * rhs.y, self * rhs.z, self * rhs.w)
    }
}

impl std::ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl std::ops::Index<usize> for Vec4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            3 => self.w,
            _ => panic!("Vec3 index out of bounds"),
        }
    }
}

impl Quat {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self{
            x,
            y,
            z,
            w,
        }
    }

    pub fn new_identity() -> Self {
        panic!("not implemented");
    }

    pub fn new_angle_axis(angle: f32, axis: &Vec3) -> Self {
        panic!("Not implemented yet");
    }

    pub fn new_from_orig_to_dest(orig: &Vec3, dest: &Vec3) -> Self {
        panic!("Not implemeneted");
    }

    pub fn inverse(&self) -> Self {
        panic!("not implemented");
    }

    pub fn slerp(a: &Self, b: &Self, t: f32) -> Self {
        panic!("Not stolen yet");
    }

    pub fn rotate_vec3(q: &Self, v: &Vec3) -> Vec3 {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<Quat> for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        panic!("Not implemented");
    }
}

impl Mat4 {
    pub fn new_translation(t: &Vec3) -> Self {
        panic!("not implemented");
    }

    pub fn new_rotation(r: &Quat) -> Self {
        panic!("not implemented");
    }

    pub fn new_uniform_scale(s: f32) -> Self {
        panic!("not implemented");
    }

    pub fn new_perspective(aspect_wh: f32, fovy: f32, near: f32, far: f32) -> Self {
        panic!("Not implemented");

        //glm::perspective_lh_zo(aspect, editmode_input.fovy, editmode_input.znear, zfar)
    }

    pub fn new_orthographic(left: f32, right: f32, bottom: f32, top: f32, znear: f32, zfar: f32) -> Self {
        panic!("Not implemented");
        //glm::ortho_lh_zo(left, right, bottom, top, znear, zfar)
    }

    pub fn new_look_at(from: &Vec3, at: &Vec3, up: &Vec3) -> Self {
        panic!("not implemented");
        //glm::look_at_lh(&self.pos_world, &(self.pos_world + self.forward_world()), &Self::up_world())
    }

    pub fn inverse(&self) -> Self {
        panic!("Not implemented");
    }

    pub fn row(&self, index: usize) -> Vec4 {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        panic!("Not implemented");
    }
}

