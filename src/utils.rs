#![allow(dead_code)]

use safewindows;
use glm::{Vec3, Vec4};

pub static PI : f32 = 3.14159265358979;

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
    return Some(t); // t may be >1.0
}

pub fn vec3_to_homogenous(vec: &Vec3, w: f32) -> Vec4 {
    return Vec4::new(vec.x, vec.y, vec.z, w);
}

pub fn fovx(fovy: f32, width: u32, height: u32) -> f32 {
    // based on:
    // (1) tan(fovy * 0.5) = 0.5h/z
    // (2) tan(fovx * 0.5) = 0.5w/z
    // rearrange (1) for z and substitute into (2) to get
    // (3) tan(fovx * 0.5) = (w/h) * tan(fovy * 0.5)

    break_assert!(false); // untested code

    let eq_3_rhs = (width as f32) / (height as f32) * (fovy * 0.5).tan();
    let half_fov_x = eq_3_rhs.atan();
    return half_fov_x * 2.0;
}