#![allow(dead_code)]

//use std::ops::{Add, Sub, Mul};

use safewindows;
use glm::{Vec3, Vec4, Quat, Mat4};
use gltf;
use std::collections::hash_map::{DefaultHasher};
use std::hash::{Hash, Hasher};

pub static PI : f32 = 3.14159265358979;

#[derive(Clone, Copy, Debug)]
pub struct STransform {
    pub t: Vec3,
    pub r: Quat,
    pub s: f32,
}

#[derive(Clone, Copy)]
pub struct SRay {
    pub origin: Vec3,
    pub dir: Vec3,
}

#[derive(Clone, Copy)]
pub struct SPlane {
    pub p: Vec3,
    pub normal: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SAABB {
    pub min: Vec3,
    pub max: Vec3,
}

pub type SHashedStr = u64;
pub fn hash_str(s: &str) -> SHashedStr {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

impl SAABB {
    pub fn new(p: &Vec3) -> Self {
        Self {
            min: p.clone(),
            max: p.clone(),
        }
    }

    pub fn new_from_points(ps: &[Vec3]) -> Self {
        let mut result = Self::new(&ps[0]);
        for pi in 1..ps.len() {
            result.expand(&ps[pi]);
        }
        result
    }

    pub fn zero() -> Self {
        Self {
            min: glm::zero(),
            max: glm::zero(),
        }
    }

    pub fn union(a: &Self, b: &Self) -> Self {
        Self{
            min: glm::min2(&a.min, &b.min),
            max: glm::max2(&a.max, &b.max),
        }
    }

    pub fn transform(aabb: &Self, b: &STransform) -> Self {
        let verts = [
            Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
        ];

        let mut result = Self::new(&b.mul_point(&verts[0]));
        for i in 1..8 {
            result.expand(&b.mul_point(&verts[i]));
        }

        result
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.max - self.min;
        return 2.0 * (d.x * d.y + d.y * d.z + d.z * d.x);
    }

    pub fn expand(&mut self, p: &Vec3) {
        self.min = glm::min2(&self.min, p);
        self.max = glm::max2(&self.max, p);
    }
}

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

pub fn lerp<T>(start: T, end: T, t: f32) -> T
where T: std::ops::Sub<Output = T> + std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T> + Copy
{
    start + (end - start) * t
}

pub fn unlerp_f32(start: f32, end: f32, cur: f32) -> f32 {
    assert!(start < end);
    (cur - start) / (end - start)
}

pub fn closest_point_on_line(line_p0: &Vec3, line_p1: &Vec3, p: &Vec3) -> (Vec3, f32) {
    let line_dir = line_p1 - line_p0;
    let line_len = glm::l2_norm(&(line_p1 - line_p0));
    let line_dir_norm = line_dir / line_len;

    let dist_along : f32 = glm::dot(&(p - line_p0), &line_dir_norm);

    let closest_pt = line_p0 + dist_along * line_dir_norm;
    (closest_pt, dist_along / line_len)
}

pub fn ray_intersects_aabb(ray: &SRay, aabb: &SAABB) -> Option<f32> {
    // Andrew Woo graphics gems 1990 (p 395-396)
    // https://web.archive.org/web/20090803054252/http://tog.acm.org/resources/GraphicsGems/gems/RayBox.c

    #[derive(PartialEq, Clone, Copy)]
    enum EQuadrant {
        None,
        Right,
        Left,
        Middle,
    };

    let mut inside = true;
    let mut quadrant = [EQuadrant::None; 3];
    let mut max_t = [0.0; 3];
    let mut candidate_plane = [0.0; 3];

    // -- find candidate planes
    for i in 0..3 {
        if ray.origin[i] < aabb.min[i] {
            quadrant[i] = EQuadrant::Left;
            candidate_plane[i] = aabb.min[i];
            inside = false;
        }
        else if ray.origin[i] > aabb.max[i] {
            quadrant[i] = EQuadrant::Right;
            candidate_plane[i] = aabb.max[i];
            inside = false;
        }
        else {
            quadrant[i] = EQuadrant::Middle;
        }
    }

    // -- ray origin inside the bounding box
    if inside {
        return Some(0.0);
    }

    // -- calculate t distances to candidate planes
    for i in 0..3 {
        if quadrant[i] != EQuadrant::Middle && ray.dir[i] != 0.0 {
            max_t[i] = (candidate_plane[i] - ray.origin[i]) / ray.dir[i];
        }
        else {
            max_t[i] = -1.0;
        }
    }

    // -- get the largest max_t for final choice of intersection
    let mut which_plane = 0;
    for i in 1..3 {
        if max_t[which_plane] < max_t[i] {
            which_plane = i;
        }
    }

    // -- check final candidate actually inside box
    if max_t[which_plane] < 0.0 {
        return None;
    }

    for i in 0..3 {
        if which_plane != i {
            let coord_i = ray.origin[i] + max_t[which_plane] * ray.dir[i];
            if coord_i < aabb.min[i] || coord_i > aabb.max[i] {
                return None;
            }
        }
    }

    return Some(max_t[which_plane]);
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

        //println!("Mul transform second {:?}", second);
        //println!("Mul transform first {:?}", first);

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

pub fn gltf_accessor_slice<'a, T>(
    accessor: &gltf::Accessor,
    expected_datatype: gltf::accessor::DataType,
    expected_dimensions: gltf::accessor::Dimensions,
    bytes: &'a Vec<u8>,
) -> &'a [T] {
    if accessor.data_type() != expected_datatype {
        println!("Expected datatype {:?}, got {:?}", expected_datatype, accessor.data_type());
        assert!(false);
    }
    assert!(accessor.dimensions() == expected_dimensions);

    let size = accessor.size();
    assert!(size == std::mem::size_of::<T>());
    let count = accessor.count();

    let view = accessor.view().unwrap();
    assert!(view.stride().is_none());

    let slice_bytes = &bytes[view.offset()..(view.offset() + size * count)];
    let (_a, result, _b) = unsafe { slice_bytes.align_to::<T>() };
    assert!(_a.len () == 0 && _b.len() == 0);

    result
}
