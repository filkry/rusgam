#![allow(dead_code)]

//use std::ops::{Add, Sub, Mul};

use safewindows;
use glm::{Vec3, Vec4, Quat, Mat4};

pub static PI : f32 = 3.14159265358979;

#[derive(Clone, Copy)]
pub struct STransform {
    pub t: Vec3,
    pub r: Quat,
    pub s: f32,
}

pub struct SRay {
    pub origin: Vec3,
    pub dir: Vec3,
}

pub struct SPlane {
    pub p: Vec3,
    pub normal: Vec3,
}

//pub fn hash64<T: Hash>(t: &T) -> u64 {
//    let mut s = DefaultHasher::new();
//    t.hash(&mut s);
//    s.finish()
//}

pub fn align_up(size: usize, align: usize) -> usize {
    if size % align == 0 {
        return size;
    }

    let result = ((size / align) + 1) * align;

    assert!(result >= size);
    assert!(result % align == 0);

    result
}

pub fn clamp<T: Copy + PartialOrd<T>>(val: T, min: T, max: T) -> T {
    if val < min {
        return min;
    }
    else if val > max {
        return max;
    }

    return val;
}

// -- $$$FRK(TODO): come back to this, not in the mood right now
/*
pub fn lerp<T: Copy + PartialOrd<T> + Add<Output=T> + Sub<Output=T> + Mul<f32>>(start: T, end: T, t: f32) -> T {
    start + t * (end - start)
}
*/

pub fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}

pub fn closest_point_on_line(line_p0: &Vec3, line_p1: &Vec3, p: &Vec3) -> (Vec3, f32) {
    let line_dir = line_p1 - line_p0;
    let line_len = glm::l2_norm(&(line_p1 - line_p0));
    let line_dir_norm = line_dir / line_len;

    let dist_along : f32 = glm::dot(&(p - line_p0), &line_dir_norm);

    let closest_pt = line_p0 + dist_along * line_dir_norm;
    (closest_pt, dist_along / line_len)
}

pub fn ray_intersects_triangle(
    ray_origin: &Vec3,
    ray_dir: &Vec3,
    t0p: &Vec3,
    t1p: &Vec3,
    t2p: &Vec3) -> Option<f32> {

    //let ray_dir_norm = ray_dir.normalized();

    // moller-trumbore from Wikipedia

    const EPSILON: f32 = 0.000_000_1;

    let edge1 = t1p - t0p;
    let edge2 = t2p - t0p;

    let h = glm::cross(&ray_dir, &edge2);
    let a = glm::dot(&edge1, &h);

    if a > -EPSILON && a < EPSILON {
        return None; // parallel
    }

    let f = 1.0 / a;
    let s = ray_origin - t0p;
    let u = f * glm::dot(&s, &h);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = glm::cross(&s, &edge1);
    let v = f * glm::dot(&ray_dir, &q);
    if v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    let t = f * glm::dot(&edge2, &q);
    if t > 0.0 {
        return Some(t); // t may be >1.0
    }

    return None;
}

impl SPlane {
    pub fn new(point_on_plane: &Vec3, plane_normal: &Vec3) -> Self {
        Self {
            p: point_on_plane.clone(),
            normal: plane_normal.clone(),
        }
    }
}

pub fn ray_plane_intersection(ray: &SRay, plane: &SPlane) -> Option<(Vec3, f32)> {
    use glm::dot;

    let denom = dot(&ray.dir, &plane.normal);

    const EPSILON: f32 = 0.000_000_1;
    if denom.abs() < EPSILON {
        return None;
    }

    let num = dot(&plane.p, &plane.normal) - dot(&ray.origin, &plane.normal);
    let t = num / denom;

    Some((ray.origin + t * ray.dir, t))
}

pub fn vec3_to_homogenous(vec: &Vec3, w: f32) -> Vec4 {
    break_assert!(w == 1.0 || w == 0.0);
    return Vec4::new(vec.x, vec.y, vec.z, w);
}

pub fn fovx(fovy: f32, width: u32, height: u32) -> f32 {
    // based on:
    // (1) tan(fovy * 0.5) = 0.5h/z
    // (2) tan(fovx * 0.5) = 0.5w/z
    // rearrange (1) for z and substitute into (2) to get
    // (3) tan(fovx * 0.5) = (w/h) * tan(fovy * 0.5)

    let eq_3_rhs = (width as f32) / (height as f32) * (fovy * 0.5).tan();
    let half_fov_x = eq_3_rhs.atan();
    return half_fov_x * 2.0;
}

impl Default for STransform {
    fn default() -> Self {
        Self {
            t: Vec3::new(0.0, 0.0, 0.0),
            r: glm::quat_identity(),
            s: 1.0,
        }
    }
}

impl STransform {
    pub fn new(t: &Vec3, r: &Quat, s: f32) -> Self {
        Self {
            t: t.clone(),
            r: r.clone(),
            s,
        }
    }

    pub fn new_translation(t: &Vec3) -> Self {
        let mut result = Self::default();
        result.t = t.clone();
        return result;
    }

    pub fn new_rotation(r: &Quat) -> Self {
        let mut result = Self::default();
        result.r = r.clone();
        return result;
    }

    pub fn inverse(&self) -> Self {
        break_assert!(self.s == 1.0); // didn't figure this out for non-1.0 scales yet
        let r_inverse = glm::quat_inverse(&self.r);

        Self {
            t: glm::quat_rotate_vec3(&r_inverse, &(-self.t)),
            r: r_inverse,
            s: 1.0,
        }
    }

    pub fn mul_transform(second: &STransform, first: &STransform) -> Self {
        break_assert!(first.s == 1.0 && second.s == 1.0); // didn't figure this out for non-1.0 scales yet

        // resulting transform is as though applying first, then second
        Self {
            t: second.t + glm::quat_rotate_vec3(&second.r, &first.t),
            r: second.r * first.r,
            s: 1.0,
        }
    }

    pub fn as_mat4(&self) -> Mat4 {
        // -- $$$FRK(TODO): could easily derivce the components of the matrix and
        // -- construct directly rather than multiplying

        let scale = glm::scaling(&Vec3::new(self.s, self.s, self.s));
        let rotation = glm::quat_to_mat4(&self.r);
        let translation = glm::translation(&self.t);

        return translation * rotation * scale;
    }

    pub fn mul_point(&self, point: &Vec3) -> Vec3 {
        return self.t + glm::quat_rotate_vec3(&self.r, &(self.s * point));
    }

    pub fn mul_vec(&self, point: &Vec3) -> Vec3 {
        return glm::quat_rotate_vec3(&self.r, &(self.s * point));
    }
}