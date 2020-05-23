
struct SMinkowskiDiffPoint {
    pos: Vec3,
    a_idx: usize,
    b_idx: usize,
}

struct SSimplex {
    pts: ArrayVec<[Vec3; 4]>,
}

pub fn support_mapping(pts: &[Vec3], dir: &Vec3) -> usize {
    let max_dist = 0.0;
    let max_idx : usize = 0;

    for (idx, cand_p) in pts.iter().enumerate() {
        let dist = glm::dot(cand_p, dir);
        if dist > max_dist {
            max_dist = dist;
            max_idx = idx;
        }
    }

    return max_idx;
}

pub fn minkowski_support_mapping(pts_a: &[Vec3], pts_b: &[Vec3], dir: &Vec3) -> SMinkowskiDiffPoint {
    let support_a = support_mapping(pts_a, dir);
    let support_b = support_mapping(pts_b, -dir);

    SMinkowskiDiffPoint{
        pos: pts_a[support_a] - pts_b[support_b],
        a_idx: support_a,
        b_idx: support_b,
    }
}

// -- third attempt, mixing sources

struct S1Simplex {
    a: Vec3,
}

struct S2Simplex {
    a: Vec3, // newest
    b: Vec3,
}

struct S3Simplex {
    a: Vec3, // newest
    b: Vec3,
    c: Vec3,
}

struct S4Simplex {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

enum ESimplex {
    One(S1Simplex),
    Two(S2Simplex),
    Three(S3Simplex),
    Four(S4Simplex),
}

pub fn update_simplex_result2(a: &Vec3, b: &Vec3, dir: Vec3) -> (ESimplex, Vec3) {
    (ESimplex::Two(
        S2Simplex{
            a: a,
            b: b,
        }),
        dir,
    )
}

impl S2Simplex {
    pub fn update_simplex(&self) -> (ESimplex, Vec3) {
        // -- three possible voronoi regions:
        // -- A, B, AB
        // -- but B can't be closest to the origin, or we wouldn't have searched in direction of A
        // -- A can't be closest to origin, or we would have found a further vert in the previous
        // -- search direction
        // -- therefore the region must be AB

        let ab = self.b - self.a;
        let ao = -self.a;

        if glm::dot(ab, ao) > 0 {
            let origin_dir_perp_ab = glm::cross(glm::cross(ab, ao), ab);
            return update_simplex_result2(self.a, self.b, origin_dir_perp_ab);
        }
        else {
            return update_simplex_result1(self.a, ao);
        }
    }
}

impl S3Simplex {
    pub fn update_simplex(&self) -> (ESimplex, Vec3) {
        // -- eight possible vorinoi regions:
        // -- A, B, C, AB, AC, BC, ABC(above), ABC(below
        // -- B, C, BC are excluded or we would not have search in direction of A
        // -- therefore the region must be A, AC, AB, ABC(above) or ABC(below)

        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let ao = -self.a;

        // -- If we are in A simplex
        if (glm::dot(a0, ab) <= 0) && (glm::dot(a0, ac) <= 0) {
            return update_simplex_result1(self.a, ao);
        }
        // -- A eliminated, AB, AC, ABC, -ABC remain

        let abc_perp = glm::cross(ab, ac);

        fn check_edge(a: &Vec3, ao: &Vec3, p: &Vec3, ap: &Vec3, abc_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            let ineq1 = glm::dot(ao, ap) >= 0.0;
            let ineq2 = glm::dot(-p, a - p) >= 0.0;
            let ineq3 = glm::dot(ao, glm::cross(ap, abc_perp)) >= 0.0;
            return ineq1 && ineq2 && ineq3;
        }

        // -- check AB
        if(check_edge(self.a, &ao, self.b, &ab, &abc_perp)) {
            let ab_perp_to_origin = glm::cross(glm::cross(ab, ao), ab);
            return update_simplex_result2(self.a, self.b, ab_perp_to_origin);
        }

        // -- check AC
        if(check_edge(self.a, &ao, self.c, &ac, &abc_perp)) {
            let ac_perp_to_origin = glm::cross(glm::cross(ac, ao), ac);
            return update_simplex_result2(self.a, self.c, ac_perp_to_origin);
        }
        // -- AB, AC eliminated, ABC, -ABC remain

        // -- need to determine if we are above ABC or below
        if glm::dot(abc_perp, ao) > 0 {
            // -- above
            update_simplex_result3(self.a, self.b, self.c, abc_perp);
        }
        else {
            // -- below
            // -- need to swizzle results, as we'll rely on the order to determine "outside"
            // -- face direction in Simplex4 case
            update_simplex_result3(self.a, self.c, self.b, -abc_perp);
        }
    }
}

impl S4Simplex {
    pub fn update_simplex(&self) -> (ESimplex, Vec3) {
        // -- possible voronoi regions:
        // -- A, B, C, D, AB, AC, AD, BC, BD, CD, ABC, ABD, ACD, BCD (no negative triangles because if it's inside we are done)
        // -- by reasoning above, only features including A remain:
        // -- A, AB, AC, AD, ABC, ABD, ACD

        // -- verifying triangle winding
        break_assert!(glm::dot(self.a - self.b, glm::cross(self.c - self.b, self.d - self.b)) > 0);

        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let ad = self.d - self.a;
        let ao = -a;

        let abc_perp = glm::cross(ab, ac);
        let abd_perp = glm::cross(ad, ab);
        let acd_perp = glm::cross(ac, ad);

        // -- check containment

        if glm::dot(abc_perp, a0) <= 0.0 && glm::dot(abd_perp, a0) <= 0.0 && glm::dot(acd_perp, ao) <= 0.0 {
            return COLLISION;
        }

        // -- If we are in A simplex
        if (glm::dot(a0, ab) <= 0) && (glm::dot(a0, ac) <= 0) && (glm::dot(a0, ad) <= 0) {
            return update_simplex_result1(self.a, ao);
        }
        // -- A eliminated, AB, AC, AD, ABC, ABD, ACD remain

        // -- here, AP must be counter-clockwise on the counter_clockwise_triangle, and clockwise
        // -- on the clockwise_triangle
        fn check_edge(a: &Vec3, ao: &Vec3, p: &Vec3, ap: &Vec3, counter_clockwise_triangle_perp: &Vec3, clockwise_triangle_perp: &Vec3) -> bool {
            // -- inequalities from Real-time collision detection
            let ineq1 = glm::dot(ao, ap) >= 0.0;
            let ineq2 = glm::dot(-p, a - p) >= 0.0;
            let ineq3 = glm::dot(ao, glm::cross(ap, counter_clockwise_triangle_perp)) >= 0.0;
            let ineq4 = glm::dot(ao, glm::cross(clockwise_triangle_perp, ap)) >= 0.0;
            return ineq1 && ineq2 && ineq3;
        }

        // -- check AB
        if check_edge(self.a, &ao, self.b, &ab, &abc_perp, &abd_perp) {
            let ab_perp_to_origin = glm::cross(glm::cross(ab, ao), ab);
            return update_simplex_result2(self.a, self.b, ab_perp_to_origin);
        }

        // -- check AC
        if check_edge(self.a, &ao, self.c, &ac, &acd_perp, &abc_perp) {
            let ac_perp_to_origin = glm::cross(glm::cross(ac, ao), ac);
            return update_simplex_result2(self.a, self.c, ac_perp_to_origin);
        }

        // -- check AD
        if check_edge(self.a, &ao, self.d, &ad, &abd_perp, &acd_perp) {
            let ad_perp_to_origin = glm::cross(glm::cross(ad, ao), ad);
            return update_simplex_result2(self.a, self.d, ad_perp_to_origin);
        }

        // -- AB, AC, AD elimineated, ABC, ABD, ACD remain

        fn check_face(a: &Vec3, ao: &Vec3, non_face_p: &Vec3, face_perp: &Vec3) -> bool {
            return (glm::dot(a0, face_perp) * glm::dot(non_face_p - a, face_perp)) < 0.0;
        }

        // -- check ABC
        if check_face(self.a, &ao, self.d, &abc_perp) {
            return update_simplex_result3(self.a, self.b, self.c, abc_perp);
        }

        // -- check ABD
        if check_face(self.a, &ao, self.c, &abd_perp) {
            return update_simplex_result3(self.a, self.b, self.d, abd_perp);
        }

        // -- check ACD
        if check_face(self.a, &ao, self.b, &acd_perp) {
            return update_simplex_result3(self.a, self.c, self.d, acd_perp);
        }
    }
}

// -- casey muratori video
// -- https://www.youtube.com/watch?v=Qupqu1xe7Io
// -- for this, we assume that the last point in the simplex is the most recent addition "A"
/*
pub fn update_simplex() {
    if simplex.pts.len() == 2 {
        let A = &simplex.pts[1];
        let B = &simplex.pts[0];

        let AB = B - A;
        let AO = -A;

        // -- first point cannot be closest to origin or we would have picked it, so just checking
        // -- if origin is in vorinoi space of edge
        if glm::dot(AB, AO) > 0 {
            dir = glm::cross(glm::cross(AB, AO), AB);
            return (simplex, dir);
        }
        else {
            simplex.pts[0] = simplex.pts[1];
            simplex.pts.pop();
            return (simplex, dir);
        }
    }

    if simplex.pts.len() == 3 {
        let a = simplex.pts[2].clone();
        let b = simplex.pts[1].clone();
        let c = simplex.pts[0].clone();

        let ab = b - a;
        let ac = c - a;
        let ao = -a;
        let abc_perp = glm::cross(ab, ac);

        let ac_perp_on_tri_plane = glm::cross(abc_perp, ac);

        if glm::dot(ac_perp_on_tri_plane, ao) > 0 { // in front of AC
            if glm::dot(ac, a0) > 0 { // in the vorinoi region of AC (not of A)
                let ac_perp_to_origin = glm::cross(glm::cross(ac, ao), ac);
                // -- keep ac
                simplex.pts[1] = a;
                simplex.pts.pop();

                return (simplex, ac_perp_to_origin);
            }
            else {
                // -- I don't understand why it's not just A here, but I'm trusting Muratori
                if glm::dot(ab, a0) > 0 { // in the vorinoi region of ab
                    let ab_perp_to_origin = glm::cross(glm::cross(ab, ao), ab);
                    // -- keep ab
                    simplex.pts[0] = a;
                    simplex.pts.pop();

                    return (simplex, ab_perp_to_origin);
                }
                else { // in vorinoi region of A
                    // -- keep a
                    simplex.pts[0] = a;
                    simplex.pts.pop();
                    simplex.pts.pop();

                    return (simplex, ao);
                }
            }
        }
        else {
            let ab_perp_on_tri_plane = glm::cross(ab, abc_perp);
            if glm::dot(ab_perp_on_tri_plane, a0) > 0 { // in front of AB
                if glm::dot(ab, a0) > 0 { // in the vorinoi region of ab
                    let ab_perp_to_origin = glm::cross(glm::cross(ab, ao), ab);
                    // -- keep ab
                    simplex.pts[0] = a;
                    simplex.pts.pop();

                    return (simplex, ab_perp_to_origin);
                }
                else { // in vorinoi region of A
                    // -- keep a
                    simplex.pts[0] = a;
                    simplex.pts.pop();
                    simplex.pts.pop();

                    return (simplex, ao);
                }
            }
            else {
                if glm::dot(abc, a0) > 0 { // in vorinoi region of ABC, in the direction of AB x AC
                    return (simplex, abc_perp);
                }
                else { // in vorinoi region of ABC, in the direction of -(AB x AB)
                    // -- swizzle to keep our triangle winding consistent (I'm not convinced this is necessary)
                    simplex.pts[0] = a;
                    simplex.pts[1] = c;
                    simplex.pts[2] = b;
                    return (simplex, -abc_perp);
                }
            }
        }
    }
}
*/

// -- from Real-time collision detection
/*
pub fn EMinimumNormPoints {
    Point(Vec3),
    Edge(Vec3, Vec3),
    Face(Vec3, Vec3, Vec3),
}


pub fn minimum_norm(simplex: &SSimplex) -> EMinimumNormPoints {
    break_assert!(simplex.set.len() > 0);

    if simplex.set.len() == 1 {
        return EMinimumNormPoints::Point(simplex.set[0]);
    }

    // -- check if the origin is in the vorinoi region for any of the vertices
    'vertex: for (i, pti) in simplex.set.iter().enumerate() {
        for (j, ptj) in simplex.set.iter().enumerate() {
            if i == j { continue; }

            if glm::dot(/*origin*/ - pti.pos, ptj.pos - pti.pos) > 0 {
                continue 'vertex;
            }
        }

        // -- all inequalities satisfied, origin is in region of pti
        return EMinimumNormPoints::Point(pti.pos);
    }

    if simple.set.len() == 2 {
        return EMinimumNormPoints::Edge(simplex.set[0].pos, simplex.set[1].pos);
    }

    // -- check if the origin is in the vorinoi region of any edge
    for i in 0..simplex.set.len() {
        for j in i..simplex.set.len() {

            if glm::dot(/*origin*/ - simplex.set[i].pos, simplex.set[j].pos - simplex.set[i].pos) < 0 {
                continue;
            }

            if glm::dot(/*origin*/ - simplex.set[j].pos, simplex.set[i].pos - simplex.set[j].pos) < 0 {
                continue;
            }
        }
    }
}
*/