// crate imports
use glm::{Vec3};

use collections::{SPoolHandle};
use entity::{SEntityBucket};
use databucket::{SDataBucket};
use render;
use render::{SRender};
use safewindows;

// Implementation of GJK
// mixed from various resources:
//     + basic algorithm structure comes from '06 Casey Mutatori GJK video
//     + inequalities for finding correct vorinoi region of simplex come from Real-Time Collision Detection


struct SMinkowskiDiffPoint {
    pos: Vec3,
    a_idx: usize,
    b_idx: usize,
}

fn support_mapping(pts: &[Vec3], dir: &Vec3) -> usize {
    let mut max_dist = 0.0;
    let mut max_idx : usize = 0;

    for (idx, cand_p) in pts.iter().enumerate() {
        let dist = glm::dot(cand_p, dir);
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

// -- used Vecs here for ease of use, since it's just a debug thing
pub struct SGJKDebug {
    has_pts: bool,
    cur_step: usize,
    steps: Vec<EGJKStepResult>,
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

        if glm::dot(&ab, &ao) > 0.0 {
            let origin_dir_perp_ab = glm::cross(&glm::cross(&ab, &ao), &ab);
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
        if (glm::dot(&ao, &ab) <= 0.0) && (glm::dot(&ao, &ac) <= 0.0) {
            return update_simplex_result1(&self.a, ao);
        }
        // -- A eliminated, AB, AC, ABC, -ABC remain

        let abc_perp = glm::cross(&ab, &ac);

        fn check_edge(a: &Vec3, ao: &Vec3, p: &Vec3, ap: &Vec3, abc_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            let ineq1 = glm::dot(&ao, &ap) >= 0.0;
            let ineq2 = glm::dot(&-p, &(a - p)) >= 0.0;
            let ineq3 = glm::dot(ao, &glm::cross(&ap, &abc_perp)) >= 0.0;
            return ineq1 && ineq2 && ineq3;
        }

        // -- check AB
        if check_edge(&self.a, &ao, &self.b, &ab, &abc_perp) {
            let ab_perp_to_origin = glm::cross(&glm::cross(&ab, &ao), &ab);
            return update_simplex_result2(&self.a, &self.b, ab_perp_to_origin);
        }

        // -- check AC
        if check_edge(&self.a, &ao, &self.c, &ac, &abc_perp) {
            let ac_perp_to_origin = glm::cross(&glm::cross(&ac, &ao), &ac);
            return update_simplex_result2(&self.a, &self.c, ac_perp_to_origin);
        }
        // -- AB, AC eliminated, ABC, -ABC remain

        // -- need to determine if we are above ABC or below
        if glm::dot(&abc_perp, &ao) > 0.0 {
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
        break_assert!(glm::dot(&(self.a - self.b), &glm::cross(&(self.c - self.b), &(self.d - self.b))) > 0.0);

        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let ad = self.d - self.a;
        let ao = -self.a;

        let abc_perp = glm::cross(&ab, &ac);
        let abd_perp = glm::cross(&ad, &ab);
        let acd_perp = glm::cross(&ac, &ad);

        // -- check containment

        if glm::dot(&abc_perp, &ao) <= 0.0 && glm::dot(&abd_perp, &ao) <= 0.0 && glm::dot(&acd_perp, &ao) <= 0.0 {
            return EGJKStepResult::Intersection;
        }

        // -- If we are in A vorinoi
        if (glm::dot(&ao, &ab) <= 0.0) && (glm::dot(&ao, &ac) <= 0.0) && (glm::dot(&ao, &ad) <= 0.0) {
            return update_simplex_result1(&self.a, ao);
        }
        // -- A eliminated, AB, AC, AD, ABC, ABD, ACD remain

        // -- here, AP must be counter-clockwise on the counter_clockwise_triangle, and clockwise
        // -- on the clockwise_triangle
        fn check_edge(a: &Vec3, ao: &Vec3, p: &Vec3, ap: &Vec3, counter_clockwise_triangle_perp: &Vec3, clockwise_triangle_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            let ineq1 = glm::dot(ao, ap) >= 0.0;
            let ineq2 = glm::dot(&-p, &(a - p)) >= 0.0;
            let ineq3 = glm::dot(&ao, &glm::cross(&ap, &counter_clockwise_triangle_perp)) >= 0.0;
            let ineq4 = glm::dot(&ao, &glm::cross(&clockwise_triangle_perp, &ap)) >= 0.0;
            return ineq1 && ineq2 && ineq3 && ineq4;
        }

        // -- check AB
        if check_edge(&self.a, &ao, &self.b, &ab, &abc_perp, &abd_perp) {
            let ab_perp_to_origin = glm::cross(&glm::cross(&ab, &ao), &ab);
            return update_simplex_result2(&self.a, &self.b, ab_perp_to_origin);
        }

        // -- check AC
        if check_edge(&self.a, &ao, &self.c, &ac, &acd_perp, &abc_perp) {
            let ac_perp_to_origin = glm::cross(&glm::cross(&ac, &ao), &ac);
            return update_simplex_result2(&self.a, &self.c, ac_perp_to_origin);
        }

        // -- check AD
        if check_edge(&self.a, &ao, &self.d, &ad, &abd_perp, &acd_perp) {
            let ad_perp_to_origin = glm::cross(&glm::cross(&ad, &ao), &ad);
            return update_simplex_result2(&self.a, &self.d, ad_perp_to_origin);
        }

        // -- AB, AC, AD elimineated, ABC, ABD, ACD remain

        fn check_face(a: &Vec3, ao: &Vec3, non_face_p: &Vec3, face_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            return (glm::dot(ao, face_perp) * glm::dot(&(non_face_p - a), face_perp)) < 0.0;
        }

        // -- check ABC
        if check_face(&self.a, &ao, &self.d, &abc_perp) {
            return update_simplex_result3(&self.a, &self.b, &self.c, abc_perp);
        }

        // -- check ABD
        if check_face(&self.a, &ao, &self.c, &abd_perp) {
            return update_simplex_result3(&self.a, &self.b, &self.d, abd_perp);
        }

        // -- check ACD
        if check_face(&self.a, &ao, &self.b, &acd_perp) {
            return update_simplex_result3(&self.a, &self.c, &self.d, acd_perp);
        }

        unreachable!();
    }
}

pub fn step_gjk(pts_a: &[Vec3], pts_b: &[Vec3], mut simplex: ESimplex, dir: &Vec3) -> EGJKStepResult {
    let a = minkowski_support_mapping(pts_a, pts_b, &dir);
    if glm::dot(&a.pos, &dir) < 0.0 {
        return EGJKStepResult::NoIntersection;
    }

    simplex.expand(&a.pos);
    simplex.update_simplex()
}

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

    break_assert!(false); // did not converge
    return false;
}

impl SGJKDebug {
    pub fn new(ctxt: &SDataBucket) -> Self {
        ctxt.get_renderer().unwrap().with_mut(|render: &mut SRender| {
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

    pub fn reset_to_entities(&mut self, ctxt: &SDataBucket, entity_1: SPoolHandle, entity_2: SPoolHandle) {
        ctxt.get_entities().unwrap().with(|entities: &SEntityBucket| {
            ctxt.get_renderer().unwrap().with_mut(|render: &mut SRender| {
                let world_verts_a = {
                    let model = entities.get_entity_model(entity_1).unwrap();
                    let loc = entities.get_entity_location(entity_1);
                    let per_vert_data = render.mesh_loader().get_per_vertex_data(model.mesh);

                    let mut world_verts = Vec::new();

                    for vd in per_vert_data.as_slice() {
                        world_verts.push(loc.mul_point(&vd.position));
                    }

                    world_verts
                };

                let world_verts_b = {
                    let model = entities.get_entity_model(entity_2).unwrap();
                    let loc = entities.get_entity_location(entity_2);
                    let per_vert_data = render.mesh_loader().get_per_vertex_data(model.mesh);

                    let mut world_verts = Vec::new();

                    for vd in per_vert_data.as_slice() {
                        world_verts.push(loc.mul_point(&vd.position));
                    }

                    world_verts
                };

                self.has_pts = true;
                self.cur_step = 0;
                self.steps.clear();
                self.pts_a = world_verts_a;
                self.pts_b = world_verts_b;
            })
        })
    }

    pub fn first_step(&self) -> EGJKStepResult {
        let s = minkowski_support_mapping(self.pts_a.as_slice(), self.pts_b.as_slice(), &Vec3::new(1.0, 1.0, 1.0));
        let simplex = ESimplex::One(S1Simplex{ a: s.pos, });
        let dir = -s.pos;

        EGJKStepResult::NewSimplexAndDir(simplex, dir)
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
                EGJKStepResult::Intersection => None,
                EGJKStepResult::NoIntersection => None,
                EGJKStepResult::NewSimplexAndDir(simplex, dir) => {
                    Some(step_gjk(&self.pts_a, &self.pts_b, simplex.clone(), dir))
                },
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
        use glm::{Vec4};

        ctxt.get_renderer().unwrap().with_mut(|render: &mut SRender| {

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
            render.temp().draw_sphere(&offset, 0.05, &Vec4::new(1.0, 0.0, 0.0, 1.0), false, Some(self.temp_render_token));

            // -- draw the minkowski difference
            let color = Vec4::new(0.0, 0.0, 1.0, 0.5);
            for diffv in minkowki_diff.as_slice() {
                let drawpt = diffv + offset;
                render.temp().draw_sphere(&drawpt, 0.05, &color, false, Some(self.temp_render_token));
            }
        });
    }
}
