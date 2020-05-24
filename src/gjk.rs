// crate imports
use glm::{Vec3};

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
            EGJKStepResult::NoIntersection => unreachable!(),
        }
    }

    break_assert!(false); // did not converge
    return false;
}

