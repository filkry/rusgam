// crate imports
use math::{Vec3};

use entity::{SEntityBucket, SEntityHandle};
use databucket::{SDataBucket};
use render;
use render::{SRender};
use imgui;

// Implementation of GJK
// mixed from various resources:
//     + basic algorithm structure comes from '06 Casey Mutatori GJK video
//     + inequalities for finding correct vorinoi region of simplex come from Real-Time Collision Detection

#[allow(dead_code)]
struct SMinkowskiDiffPoint {
    pos: Vec3,
    a_idx: usize,
    b_idx: usize,
}

fn support_mapping(pts: &[Vec3], dir: &Vec3) -> usize {
    let mut max_dist = std::f32::NEG_INFINITY;
    let mut max_idx : usize = 0;

    for (idx, cand_p) in pts.iter().enumerate() {
        let dist = Vec3::dot(cand_p, dir);
        if dist > max_dist {
            max_dist = dist;
            max_idx = idx;
        }
    }

    return max_idx;
}

fn minkowski_support_mapping(pts_a: &[Vec3], pts_b: &[Vec3], dir: &Vec3) -> SMinkowskiDiffPoint {
    let support_a = support_mapping(pts_a, dir);
    let support_b = support_mapping(pts_b, &-dir);

    SMinkowskiDiffPoint{
        pos: pts_a[support_a] - pts_b[support_b],
        a_idx: support_a,
        b_idx: support_b,
    }
}

#[derive(Clone)]
pub struct S1Simplex {
    a: Vec3,
}

#[derive(Clone)]
pub struct S2Simplex {
    a: Vec3, // newest
    b: Vec3,
}

#[derive(Clone)]
pub struct S3Simplex {
    a: Vec3, // newest
    b: Vec3,
    c: Vec3,
}

#[derive(Clone)]
pub struct S4Simplex {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
}

pub enum EGJKStepResult {
    NoIntersection,
    Intersection,
    NewSimplexAndDir(ESimplex, Vec3),
}

#[derive(Clone)]
pub enum ESimplex {
    One(S1Simplex),
    Two(S2Simplex),
    Three(S3Simplex),
    Four(S4Simplex),
}

#[allow(dead_code)]
enum EGJKDebugStep {
    NoIntersection,
    Intersection,
    Expand(ESimplex),
    UpdateSimplex(EGJKStepResult),
}

// -- used Vecs here for ease of use, since it's just a debug thing
#[allow(dead_code)]
pub struct SGJKDebug {
    has_pts: bool,
    cur_step: usize,
    steps: Vec<EGJKDebugStep>,
    pts_a: Vec<Vec3>,
    pts_b: Vec<Vec3>,
    temp_render_token: render::temp::SToken,
}

impl ESimplex {
    pub fn expand(&mut self, a: &Vec3) {
        *self = match self {
            Self::One(simplex) => Self::Two(simplex.expand(a)),
            Self::Two(simplex) => Self::Three(simplex.expand(a)),
            Self::Three(simplex) => Self::Four(simplex.expand(a)),
            Self::Four(_) => unreachable!(),
        }
    }

    pub fn update_simplex(&self) -> EGJKStepResult {
        match self {
            Self::One(_) => unreachable!(),
            Self::Two(simplex) => simplex.update_simplex(),
            Self::Three(simplex) => simplex.update_simplex(),
            Self::Four(simplex) => simplex.update_simplex(),
        }
    }
}

fn update_simplex_result1(a: &Vec3, dir: Vec3) -> EGJKStepResult {
    EGJKStepResult::NewSimplexAndDir(ESimplex::One(
        S1Simplex{
            a: a.clone(),
        }),
        dir,
    )
}

fn update_simplex_result2(a: &Vec3, b: &Vec3, dir: Vec3) -> EGJKStepResult {
    EGJKStepResult::NewSimplexAndDir(ESimplex::Two(
        S2Simplex{
            a: a.clone(),
            b: b.clone(),
        }),
        dir,
    )
}

fn update_simplex_result3(a: &Vec3, b: &Vec3, c: &Vec3, dir: Vec3) -> EGJKStepResult {
    EGJKStepResult::NewSimplexAndDir(ESimplex::Three(
        S3Simplex{
            a: a.clone(),
            b: b.clone(),
            c: c.clone(),
        }),
        dir,
    )
}

impl S1Simplex {
    fn expand(&self, a: &Vec3) -> S2Simplex {
        S2Simplex{
            a: a.clone(),
            b: self.a,
        }
    }
}

impl S2Simplex {
    fn update_simplex(&self) -> EGJKStepResult {
        // -- three possible voronoi regions:
        // -- A, B, AB
        // -- but B can't be closest to the origin, or we wouldn't have searched in direction of A
        // -- A can't be closest to origin, or we would have found a further vert in the previous
        // -- search direction
        // -- therefore the region must be AB

        let ab = self.b - self.a;
        let ao = -self.a;

        if Vec3::dot(&ab, &ao) > 0.0 {
            let origin_dir_perp_ab = Vec3::cross(&Vec3::cross(&ab, &ao), &ab);
            return update_simplex_result2(&self.a, &self.b, origin_dir_perp_ab);
        }
        else {
            return update_simplex_result1(&self.a, ao);
        }
    }

    fn expand(&self, a: &Vec3) -> S3Simplex {
        S3Simplex{
            a: a.clone(),
            b: self.a,
            c: self.b,
        }
    }
}

impl S3Simplex {
    fn update_simplex(&self) -> EGJKStepResult {
        // -- eight possible vorinoi regions:
        // -- A, B, C, AB, AC, BC, ABC(above), ABC(below
        // -- B, C, BC are excluded or we would not have search in direction of A
        // -- therefore the region must be A, AC, AB, ABC(above) or ABC(below)

        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let ao = -self.a;

        // -- If we are in A vorinoi
        if (Vec3::dot(&ao, &ab) <= 0.0) && (Vec3::dot(&ao, &ac) <= 0.0) {
            return update_simplex_result1(&self.a, ao);
        }
        // -- A eliminated, AB, AC, ABC, -ABC remain

        let abc_perp = Vec3::cross(&ab, &ac);

        // -- ap must be counter-clockwise on the triangle defining abc_perp
        fn check_edge(a: &Vec3, ao: &Vec3, p: &Vec3, ap: &Vec3, abc_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            let ineq1 = Vec3::dot(&ao, &ap) >= 0.0;
            let ineq2 = Vec3::dot(&-p, &(a - p)) >= 0.0;
            let ineq3 = Vec3::dot(ao, &Vec3::cross(&ap, &abc_perp)) >= 0.0;
            return ineq1 && ineq2 && ineq3;
        }

        // -- check AB
        if check_edge(&self.a, &ao, &self.b, &ab, &abc_perp) {
            let ab_perp_to_origin = Vec3::cross(&Vec3::cross(&ab, &ao), &ab);
            return update_simplex_result2(&self.a, &self.b, ab_perp_to_origin);
        }

        // -- check AC
        // -- negate the normal because AC is clockwise on a counter-clockwise ABC
        if check_edge(&self.a, &ao, &self.c, &ac, &-abc_perp) {
            let ac_perp_to_origin = Vec3::cross(&Vec3::cross(&ac, &ao), &ac);
            return update_simplex_result2(&self.a, &self.c, ac_perp_to_origin);
        }
        // -- AB, AC eliminated, ABC, -ABC remain

        // -- need to determine if we are above ABC or below
        if Vec3::dot(&abc_perp, &ao) > 0.0 {
            // -- above
            return update_simplex_result3(&self.a, &self.b, &self.c, abc_perp);
        }
        else {
            // -- below
            // -- need to swizzle results, as we'll rely on the order to determine "outside"
            // -- face direction in Simplex4 case
            return update_simplex_result3(&self.a, &self.c, &self.b, -abc_perp);
        }
    }

    fn expand(&self, a: &Vec3) -> S4Simplex {
        S4Simplex{
            a: a.clone(),
            b: self.a,
            c: self.b,
            d: self.c,
        }
    }
}

impl S4Simplex {
    fn update_simplex(&self) -> EGJKStepResult {
        // -- possible voronoi regions:
        // -- A, B, C, D, AB, AC, AD, BC, BD, CD, ABC, ABD, ACD, BCD (no negative triangles because if it's inside we are done)
        // -- by reasoning above, only features including A remain:
        // -- A, AB, AC, AD, ABC, ABD, ACD

        // -- verifying triangle winding
        //break_assert!(dot(&(self.a - self.b), &cross(&(self.c - self.b), &(self.d - self.b))) > 0.0);

        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let ad = self.d - self.a;
        let ao = -self.a;

        let abc_perp = Vec3::cross(&ab, &ac);
        let abd_perp = Vec3::cross(&ad, &ab);
        let acd_perp = Vec3::cross(&ac, &ad);

        // -- check containment
        let inside_abc = Vec3::dot(&abc_perp, &ao) <= 0.0;
        let inside_abd = Vec3::dot(&abd_perp, &ao) <= 0.0;
        let inside_acd = Vec3::dot(&acd_perp, &ao) <= 0.0;

        if inside_abc && inside_abd && inside_acd {
            return EGJKStepResult::Intersection;
        }

        // -- If we are in A vorinoi
        if (Vec3::dot(&ao, &ab) <= 0.0) && (Vec3::dot(&ao, &ac) <= 0.0) && (Vec3::dot(&ao, &ad) <= 0.0) {
            return update_simplex_result1(&self.a, ao);
        }
        // -- A eliminated, AB, AC, AD, ABC, ABD, ACD remain

        // -- here, AP must be counter-clockwise on the counter_clockwise_triangle, and clockwise
        // -- on the clockwise_triangle
        fn check_edge(a: &Vec3, ao: &Vec3, p: &Vec3, ap: &Vec3, counter_clockwise_triangle_perp: &Vec3, clockwise_triangle_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            let ineq1 = Vec3::dot(ao, ap) >= 0.0;
            let ineq2 = Vec3::dot(&-p, &(a - p)) >= 0.0;
            let ineq3 = Vec3::dot(&ao, &Vec3::cross(&ap, &counter_clockwise_triangle_perp)) >= 0.0;
            let ineq4 = Vec3::dot(&ao, &Vec3::cross(&clockwise_triangle_perp, &ap)) >= 0.0;
            return ineq1 && ineq2 && ineq3 && ineq4;
        }

        // -- check AB
        if check_edge(&self.a, &ao, &self.b, &ab, &abc_perp, &abd_perp) {
            let ab_perp_to_origin = Vec3::cross(&Vec3::cross(&ab, &ao), &ab);
            return update_simplex_result2(&self.a, &self.b, ab_perp_to_origin);
        }

        // -- check AC
        if check_edge(&self.a, &ao, &self.c, &ac, &acd_perp, &abc_perp) {
            let ac_perp_to_origin = Vec3::cross(&Vec3::cross(&ac, &ao), &ac);
            return update_simplex_result2(&self.a, &self.c, ac_perp_to_origin);
        }

        // -- check AD
        if check_edge(&self.a, &ao, &self.d, &ad, &abd_perp, &acd_perp) {
            let ad_perp_to_origin = Vec3::cross(&Vec3::cross(&ad, &ao), &ad);
            return update_simplex_result2(&self.a, &self.d, ad_perp_to_origin);
        }

        // -- AB, AC, AD elimineated, ABC, ABD, ACD remain

        fn check_face(a: &Vec3, ao: &Vec3, non_face_p: &Vec3, face_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            return (Vec3::dot(ao, face_perp) * Vec3::dot(&(non_face_p - a), face_perp)) < 0.0;
        }

        // -- check ABC
        if check_face(&self.a, &ao, &self.d, &abc_perp) {
            return update_simplex_result3(&self.a, &self.b, &self.c, abc_perp);
        }

        // -- check ABD
        if check_face(&self.a, &ao, &self.c, &abd_perp) {
            // -- swizzle here to ensure counter-clockwise
            return update_simplex_result3(&self.a, &self.d, &self.b, abd_perp);
        }

        // -- check ACD
        if check_face(&self.a, &ao, &self.b, &acd_perp) {
            return update_simplex_result3(&self.a, &self.c, &self.d, acd_perp);
        }

        // -- $$$FRK(TODO): return an Err here, then use the debugger to see what's happening
        // -- in more detail
        // -- numerical issues, return any face the point is on the correct side of
        if Vec3::dot(&abc_perp, &ao) > 0.0 {
            return update_simplex_result3(&self.a, &self.b, &self.c, abc_perp);
        }
        else if Vec3::dot(&abd_perp, &ao) > 0.0 {
            return update_simplex_result3(&self.a, &self.b, &self.d, abd_perp);
        }
        else if Vec3::dot(&acd_perp, &ao) > 0.0 {
            return update_simplex_result3(&self.a, &self.c, &self.d, acd_perp);
        }

        unreachable!();
    }
}

pub fn step_gjk(pts_a: &[Vec3], pts_b: &[Vec3], mut simplex: ESimplex, dir: &Vec3) -> EGJKStepResult {
    let a = minkowski_support_mapping(pts_a, pts_b, &dir);
    if Vec3::dot(&a.pos, &dir) < 0.0 {
        return EGJKStepResult::NoIntersection;
    }

    simplex.expand(&a.pos);
    simplex.update_simplex()
}

#[allow(dead_code)]
pub fn gjk(pts_a: &[Vec3], pts_b: &[Vec3]) -> bool {
    let s = minkowski_support_mapping(pts_a, pts_b, &Vec3::new(1.0, 1.0, 1.0));
    let mut simplex = ESimplex::One(S1Simplex{ a: s.pos, });
    let mut dir = -s.pos;

    let max_iters = 64;
    for _ in 0..max_iters {
        match step_gjk(pts_a, pts_b, simplex.clone(), &dir) {
            EGJKStepResult::Intersection => {
                return true;
            },
            EGJKStepResult::NewSimplexAndDir(new_simplex, new_dir) => {
                simplex = new_simplex;
                dir = new_dir;
            },
            EGJKStepResult::NoIntersection => {
                return false;
            },
        }
    }

    println!("ERROR: Minkowski did not converge.");
    return false;
}

#[allow(dead_code)]
impl SGJKDebug {
    pub fn new(ctxt: &SDataBucket) -> Self {
        ctxt.get_renderer().with_mut(|render: &mut SRender| {
            Self {
                has_pts: false,
                cur_step: 0,
                steps: Vec::new(),
                pts_a: Vec::new(),
                pts_b: Vec::new(),
                temp_render_token: render.temp().get_token(),
            }
        })
    }

    pub fn reset_to_entities(&mut self, ctxt: &SDataBucket, entity_1: SEntityHandle, entity_2: SEntityHandle) {
        use entity_model;

        ctxt.get::<SEntityBucket>()
            .and::<render::SRender>()
            .and::<entity_model::SBucket>()
            .with_ccc(|entities, render, em| {
                let world_verts_a = {
                    let e1_model_handle = em.handle_for_entity(entity_1).unwrap();
                    let model = em.get_model(e1_model_handle);

                    let loc = entities.get_entity_location(entity_1);
                    let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                    let mut world_verts = Vec::new();

                    for v in mesh_local_vs.as_slice() {
                        world_verts.push(loc.mul_point(&v));
                    }

                    world_verts
                };

                let world_verts_b = {
                    let e2_model_handle = em.handle_for_entity(entity_2).unwrap();
                    let model = em.get_model(e2_model_handle);
                    let loc = entities.get_entity_location(entity_2);
                    let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                    let mut world_verts = Vec::new();

                    for v in mesh_local_vs.as_slice() {
                        world_verts.push(loc.mul_point(&v));
                    }

                    world_verts
                };

                self.has_pts = true;
                self.cur_step = 0;
                self.steps.clear();
                self.pts_a = world_verts_a;
                self.pts_b = world_verts_b;
            })
    }

    fn first_step(&self) -> EGJKDebugStep {
        let s = minkowski_support_mapping(self.pts_a.as_slice(), self.pts_b.as_slice(), &Vec3::new(1.0, 1.0, 1.0));
        let simplex = ESimplex::One(S1Simplex{ a: s.pos, });
        let dir = -s.pos;

        EGJKDebugStep::UpdateSimplex(EGJKStepResult::NewSimplexAndDir(simplex, dir))
    }

    pub fn step_backward(&mut self) {
        if self.cur_step > 0 {
            self.cur_step -= 1;
        }
    }

    pub fn step_forward(&mut self) {
        if self.steps.len() > 0 && self.cur_step < (self.steps.len() - 1) {
            self.cur_step += 1;
        }
        else {
            let last_step_result = {
                if self.steps.len() == 0 {
                    let first_step = self.first_step();
                    self.steps.push(first_step);
                }
                self.steps.last().unwrap()
            };
            let next_step_result = match last_step_result {
                EGJKDebugStep::Intersection => None,
                EGJKDebugStep::NoIntersection => None,
                EGJKDebugStep::Expand(simplex) => {
                    // -- step after expand is update
                    let step_result = simplex.update_simplex();

                    Some(match step_result {
                        EGJKStepResult::Intersection => EGJKDebugStep::Intersection,
                        EGJKStepResult::NoIntersection => EGJKDebugStep::NoIntersection,
                        EGJKStepResult::NewSimplexAndDir(_, _) => EGJKDebugStep::UpdateSimplex(step_result),
                    })
                },
                EGJKDebugStep::UpdateSimplex(gjk_step_result) => {
                    // -- step after update is expand
                    if let EGJKStepResult::NewSimplexAndDir(simplex, dir) = gjk_step_result {
                        let mut new_simplex = simplex.clone();
                        let a = minkowski_support_mapping(&self.pts_a, &self.pts_b, &dir);
                        if Vec3::dot(&a.pos, &dir) < 0.0 {
                            Some(EGJKDebugStep::NoIntersection)
                        }
                        else {
                            new_simplex.expand(&a.pos);
                            Some(EGJKDebugStep::Expand(new_simplex))
                        }
                    }
                    else {
                        None
                    }
                }
            };

            if let Some(result) = next_step_result {
                self.steps.push(result);

                if self.steps.len() > 1 { // handle pushing the very first step
                    self.cur_step += 1;
                }
            }
        }
    }

    pub fn render_cur_step(&self, ctxt: &SDataBucket) {
        use math::{Vec4};

        ctxt.get_renderer().with_mut(|render: &mut SRender| {

            let tok = Some(self.temp_render_token);

            let offset = Vec3::new(0.0, 4.0, 0.0);

            // -- clear old drawings
            render.temp().clear_token(self.temp_render_token);

            let mut minkowki_diff = Vec::new();
            for v1 in self.pts_a.as_slice() {
                for v2 in self.pts_b.as_slice() {
                    minkowki_diff.push(v1 - v2);
                }
            }

            // -- draw the origin
            render.temp().draw_sphere(&offset, 0.05, &Vec4::new(1.0, 0.0, 0.0, 1.0), false, tok);

            // -- draw the minkowski difference
            let color = Vec4::new(0.0, 0.0, 1.0, 0.5);
            for diffv in minkowki_diff.as_slice() {
                let drawpt = diffv + offset;
                render.temp().draw_sphere(&drawpt, 0.05, &color, false, tok);
            }

            // -- draw the simplex
            let simplex_color = Vec4::new(0.0, 1.0, 0.0, 0.5);
            let dir_color = Vec4::new(1.0, 1.0, 1.0, 0.5);
            let normal_color = Vec4::new(0.0, 0.0, 1.0, 0.5);
            let ao_color = Vec4::new(1.0, 1.0, 1.0, 0.5);
            let (simplex_to_draw, dir_to_draw) = match &self.steps[self.cur_step] {
                EGJKDebugStep::NoIntersection => (None, None),
                EGJKDebugStep::Intersection => (None, None),
                EGJKDebugStep::Expand(simplex) => (Some(simplex.clone()), None),
                EGJKDebugStep::UpdateSimplex(step_result) => {
                    if let EGJKStepResult::NewSimplexAndDir(simplex, dir) = step_result {
                        (Some(simplex.clone()), Some(dir))
                    }
                    else {
                        (None, None)
                    }
                }
            };

            if let Some(simplex) = simplex_to_draw {
                let a = match simplex {
                    ESimplex::One(internal) => {
                        render.temp().draw_sphere(&(internal.a + offset), 0.06, &simplex_color, false, tok);
                        internal.a.clone()
                    },
                    ESimplex::Two(internal) => {
                        render.temp().draw_sphere(&(internal.a + offset), 0.1, &Vec4::new(1.0, 1.0, 1.0, 1.0), false, tok);
                        render.temp().draw_sphere(&(internal.b + offset), 0.1, &Vec4::new(1.0, 0.0, 0.0, 1.0), false, tok);
                        render.temp().draw_line(&(internal.a + offset), &(internal.b + offset), &simplex_color, false, tok);
                        internal.a.clone()
                    },
                    ESimplex::Three(internal) => {
                        render.temp().draw_sphere(&(internal.a + offset), 0.1, &Vec4::new(1.0, 1.0, 1.0, 1.0), false, tok);
                        render.temp().draw_sphere(&(internal.b + offset), 0.1, &Vec4::new(1.0, 0.0, 0.0, 1.0), false, tok);
                        render.temp().draw_sphere(&(internal.c + offset), 0.1, &Vec4::new(0.0, 1.0, 0.0, 1.0), false, tok);
                        render.temp().draw_line(&(internal.a + offset), &(internal.b + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.b + offset), &(internal.c + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.c + offset), &(internal.a + offset), &simplex_color, false, tok);
                        internal.a.clone()
                    },
                    ESimplex::Four(internal) => {
                        // -- draw points
                        render.temp().draw_sphere(&(internal.a + offset), 0.1, &Vec4::new(1.0, 1.0, 1.0, 1.0), false, tok);
                        render.temp().draw_sphere(&(internal.b + offset), 0.1, &Vec4::new(1.0, 0.0, 0.0, 1.0), false, tok);
                        render.temp().draw_sphere(&(internal.c + offset), 0.1, &Vec4::new(0.0, 1.0, 0.0, 1.0), false, tok);
                        render.temp().draw_sphere(&(internal.d + offset), 0.1, &Vec4::new(0.0, 0.0, 1.0, 1.0), false, tok);

                        // -- draw lines
                        render.temp().draw_line(&(internal.a + offset), &(internal.b + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.a + offset), &(internal.c + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.a + offset), &(internal.d + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.b + offset), &(internal.c + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.b + offset), &(internal.d + offset), &simplex_color, false, tok);
                        render.temp().draw_line(&(internal.c + offset), &(internal.d + offset), &simplex_color, false, tok);

                        // -- draw normals
                        // -- copy/pasted from the code, if that changed, this becomes inaccurate
                        let ab = internal.b - internal.a;
                        let ac = internal.c - internal.a;
                        let ad = internal.d - internal.a;
                        let abc_perp = Vec3::cross(&ab, &ac);
                        let abd_perp = Vec3::cross(&ad, &ab);
                        let acd_perp = Vec3::cross(&ac, &ad);
                        let abc_centroid = (1.0 / 3.0) * (internal.a + internal.b + internal.c);
                        let abd_centroid = (1.0 / 3.0) * (internal.a + internal.b + internal.d);
                        let acd_centroid = (1.0 / 3.0) * (internal.a + internal.c + internal.d);
                        render.temp().draw_line(&(abc_centroid + offset), &(abc_centroid + offset + abc_perp), &normal_color, false, tok);
                        render.temp().draw_line(&(abd_centroid + offset), &(abd_centroid + offset + abd_perp), &normal_color, false, tok);
                        render.temp().draw_line(&(acd_centroid + offset), &(acd_centroid + offset + acd_perp), &normal_color, false, tok);

                        // -- draw ao
                        render.temp().draw_line(&(offset), &(internal.a + offset), &ao_color, false, tok);

                        internal.a.clone()
                    },
                };

                if let Some(dir) = dir_to_draw {
                    render.temp().draw_line(&(a + offset), &(a + offset + dir), &dir_color, false, tok);
                }
            }
        });
    }

    pub fn imgui_menu(&mut self, imgui_ui: &imgui::Ui, ctxt: &SDataBucket, entity_1: Option<SEntityHandle>, entity_2: Option<SEntityHandle>) {
        use imgui::*;

        imgui_ui.menu(im_str!("GJK"), true, || {
            if let Some(e1) = entity_1 {
                if let Some(e2) = entity_2 {
                    if imgui_ui.small_button(im_str!("Start")) {
                        self.reset_to_entities(ctxt, e1, e2);
                        self.step_forward();
                        self.render_cur_step(ctxt);
                    }
                }
            }

            imgui_ui.text(&im_str!("Active test: {}", self.has_pts));
            if self.has_pts {
                imgui_ui.text(&im_str!("Cur step: {}/{}", self.cur_step, self.steps.len() - 1));
                let step_type_str = match self.steps[self.cur_step] {
                    EGJKDebugStep::NoIntersection => "No intersection",
                    EGJKDebugStep::Intersection => "Intersection",
                    EGJKDebugStep::Expand(_) => "Expand",
                    EGJKDebugStep::UpdateSimplex(_) => "Update simplex",
                };
                imgui_ui.text(&im_str!("Step type: {}", step_type_str));
                if imgui_ui.small_button(im_str!("Step forward")) {
                    self.step_forward();
                    self.render_cur_step(ctxt);
                }
                if imgui_ui.small_button(im_str!("Step forward (recompute)")) {
                    while self.steps.len() > (self.cur_step + 1) {
                        self.steps.pop();
                    }

                    self.step_forward();
                    self.render_cur_step(ctxt);
                }
                if imgui_ui.small_button(im_str!("Step backward")) {
                    self.step_backward();
                    self.render_cur_step(ctxt);
                }
            }
        });
    }
}
