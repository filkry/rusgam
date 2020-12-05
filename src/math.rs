// -- I still don't want to implement my own types, but I'm tired of how gnarly glm types look
// -- in the debugger. So I'm going to try creating memory-layout-equivalent types, and unsafely
// -- casting to glm types in my operations
extern crate nalgebra_glm as glm;
use utils;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec2{
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec3{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec4{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Quat{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Copy, Clone, Debug)]
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
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn is_normalized(&self) -> bool {
        panic!("Not implemented");
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

    pub fn cross(a: &Self, b: &Self) -> Self {
        let x = a.y * b.z - a.z * b.y;
        let y = a.x * b.z - a.z * b.x;
        let z = a.x * b.y - a.y * b.x;
        Self::new(x, y, z)
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
        // -- a dot b = |a| * |b| * cos(theta)
        // -- cos(theta) = a dot b / |a||b|
        // -- theta = acos(a dot b / |a||b|)

        let rhs = Self::dot(a, b) / (a.mag() * b.mag());
        rhs.acos()
    }

    pub fn rotate_y(&self, angle: f32) -> Self {
        unsafe {
            let glm_t = (self as *const Vec3 as *const glm::Vec3).as_ref().expect("catastrophic failure");
            let glm_res = glm::rotate_y_vec3(glm_t, angle);

            *((&glm_res).as_ptr() as *const Vec3)
        }
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

impl std::ops::Add<&Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Add<Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Add<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: &Vec3) -> Self::Output {
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
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vec3 index out of bounds"),
        }
    }
}

impl std::ops::IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
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
        Self::new(0.0, 0.0, 0.0, 0.0)
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
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Vec3 index out of bounds"),
        }
    }
}

impl std::ops::IndexMut<usize> for Vec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Vec4 index out of bounds"),
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
        assert!(axis.is_normalized());

        let half_angle = angle / 2.0;
        let sin_half_angle = half_angle.sin();

        Self::new(
            axis.x * sin_half_angle,
            axis.y * sin_half_angle,
            axis.z * sin_half_angle,
            half_angle.cos(),
        )
    }

    pub fn new_from_orig_to_dest(orig: &Vec3, dest: &Vec3) -> Self {
        unsafe {
            let orig_glm_t = (orig as *const Vec3 as *const glm::Vec3).as_ref().expect("");
            let dest_glm_t = (dest as *const Vec3 as *const glm::Vec3).as_ref().expect("");
            let glm_res = glm::quat_rotation(orig_glm_t, dest_glm_t);

            std::mem::transmute::<glm::Quat, Quat>(glm_res)
        }
    }

    pub fn inverse(&self) -> Self {
        panic!("not implemented");
    }

    pub fn slerp(a: &Self, b: &Self, t: f32) -> Self {
        unsafe {
            let a_glm_t = (a as *const Quat as *const glm::Quat).as_ref().expect("");
            let b_glm_t = (b as *const Quat as *const glm::Quat).as_ref().expect("");
            let glm_res = glm::quat_slerp(a_glm_t, b_glm_t, t);

            std::mem::transmute::<glm::Quat, Quat>(glm_res)
        }
    }

    pub fn rotate_vec3(q: &Self, v: &Vec3) -> Vec3 {
        unsafe {
            let q_glm_t = (q as *const Quat as *const glm::Quat).as_ref().expect("");
            let v_glm_t = (v as *const Vec3 as *const glm::Vec3).as_ref().expect("");
            let glm_res = glm::quat_rotate_vec3(q_glm_t, v_glm_t);

            std::mem::transmute::<glm::Vec3, Vec3>(glm_res)
        }
    }
}

impl std::ops::Mul<Quat> for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let self_glm_t = &self as *const Quat as *const glm::Quat;
            let rhs_glm_t = &rhs as *const Quat as *const glm::Quat;

            let glm_res = *self_glm_t * *rhs_glm_t;
            std::mem::transmute::<glm::Quat, Quat>(glm_res)
        }
    }
}

impl Mat4 {
    pub fn new_identity() -> Self {
        let mut result = Self {
            data: [0.0; 16],
        };
        result[0][0] = 1.0;
        result[1][1] = 1.0;
        result[2][2] = 1.0;
        result[3][3] = 1.0;

        result
    }

    pub fn new_translation(t: &Vec3) -> Self {
        let mut result = Self::new_identity();
        result[3][0] = t.x;
        result[3][1] = t.y;
        result[3][2] = t.z;

        result
    }

    pub fn new_rotation(r: &Quat) -> Self {
        unsafe {
            let r_glm_t = r as *const Quat as *const glm::Quat;
            let glm_res = glm::quat_to_mat4(r_glm_t.as_ref().expect(""));
            std::mem::transmute::<glm::Mat4, Mat4>(glm_res)
        }
    }

    pub fn new_uniform_scale(s: f32) -> Self {
        let mut result = Self::new_identity();
        result[0][0] = s;
        result[1][1] = s;
        result[2][2] = s;
        result
    }

    pub fn new_perspective(aspect_wh: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        let glm_res = glm::perspective_lh_zo(aspect_wh, fovy, znear, zfar);
        unsafe {
            std::mem::transmute::<glm::Mat4, Mat4>(glm_res)
        }
    }

    pub fn new_orthographic(left: f32, right: f32, bottom: f32, top: f32, znear: f32, zfar: f32) -> Self {
        let glm_res = glm::ortho_lh_zo(left, right, bottom, top, znear, zfar);
        unsafe {
            std::mem::transmute::<glm::Mat4, Mat4>(glm_res)
        }
    }

    pub fn new_look_at(from: &Vec3, at: &Vec3, up: &Vec3) -> Self {
        let from_glm_t = from as *const Vec3 as *const glm::Vec3;
        let at_glm_t = at as *const Vec3 as *const glm::Vec3;
        let up_glm_t = up as *const Vec3 as *const glm::Vec3;

        unsafe {
            let glm_res = glm::look_at_lh(
                from_glm_t.as_ref().expect(""),
                at_glm_t.as_ref().expect(""),
                up_glm_t.as_ref().expect(""),
            );
            std::mem::transmute::<glm::Mat4, Mat4>(glm_res)
        }
    }

    pub fn inverse(&self) -> Self {
        panic!("Not implemented");
    }

    pub fn row(&self, index: usize) -> Vec4 {
        Vec4::new(
            self[0][index],
            self[1][index],
            self[2][index],
            self[3][index],
        )
    }
}

impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<&Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<Mat4> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<&Mat4> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        panic!("Not implemented");
    }
}

impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        panic!("Not implemented");
    }
}

impl std::ops::Index<usize> for Mat4 {
    type Output = [f32];

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.data[0..4],
            1 => &self.data[4..8],
            2 => &self.data[8..12],
            3 => &self.data[12..16],
            _ => panic!("Mat4 index out of bounds"),
        }
    }
}

impl std::ops::IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.data[0..4],
            1 => &mut self.data[4..8],
            2 => &mut self.data[8..12],
            3 => &mut self.data[12..16],
            _ => panic!("Mat4 index out of bounds"),
        }
    }
}

pub fn validate_glm_compatibility() {
    use std::mem::{size_of};

    // -- Vec3
    {
        assert!(size_of::<Vec3>() == size_of::<glm::Vec3>());
        let my_vec = Vec3::new(123.45, 8236.11111, 329.0);
        let glm_vec = glm::Vec3::new(123.45, 8236.11111, 329.0);
        let my_vec_as_glm_vec = &my_vec as *const Vec3 as *const glm::Vec3;
        unsafe {
            assert!(*my_vec_as_glm_vec == glm_vec);
        }
    }

    // -- Quat
    {
        assert!(size_of::<Quat>() == size_of::<glm::Quat>());
        let my_quat = Quat::new(123.45, 8236.11111, 329.0, 99999.0);
        let glm_quat = glm::Quat::new(123.45, 8236.11111, 329.0, 99999.0);
        let my_quat_as_glm_quat = &my_quat as *const Quat as *const glm::Quat;
        unsafe {
            assert!(*my_quat_as_glm_quat == glm_quat);
        }
    }

    // -- Mat4
    {
        assert!(size_of::<Mat4>() == size_of::<glm::Mat4>());
        let mut my_mat4 = Mat4::new_identity();
        let mut glm_mat4 : glm::Mat4 = glm::identity();

        let vals : [f32; 16] = [
            23812.423, 777.2, 10.0, 23333.1111,
            123.455, -0.23, -99.0, 238183.33,
            -9999.2222, -1.0, 0.0, 56.0,
            5.0, 7.0, 8.0, 10.0,
        ];
        for i in 1..16 {
            my_mat4[i / 4][i % 4] = vals[i];
            glm_mat4[i] = vals[i];
        }

        let my_mat4_as_glm_mat4 = &my_mat4 as *const Mat4 as *const glm::Mat4;
        unsafe {
            assert!(*my_mat4_as_glm_mat4 == glm_mat4);
        }
    }
}