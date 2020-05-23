
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
        let ao = -a;

        // -- If we are in A simplex
        if (glm::dot(a0, ab) <= 0) && (glm::dot(a0, ac) <= 0) {
            return update_simplex_result1(self.a, ao);
        }

        let abc_perp = glm::cross(ab, ac);

        let ac_perp_on_tri_plane = glm::cross(abc_perp, ac);
        let ab_perp_on_tri_plane = glm::cross(ab, abc_perp);

        if glm::dot(ac_perp_on_tri_plane, a0) > 0  {
            // -- excludes ABC, since this is effectively a plane side test for one plane of the
            // -- triangles's corresponding prism

            // -- I think this excludes AB, since if a point was outside the planes AB and AC, it
            // -- would necessarily be further along the previous search direction than A
            break_assert!(glm::dot(ab_perp_on_tri_plane, a0) <= 0);

            // -- thus, the result must be AC
            let ac_perp_to_origin = glm::cross(glm::cross(ac, ao), ac);

            update_simplex_result2(self.a, self.c, ac_perp_to_origin)
        }
        // -- AC excluded beyond here
        else if glm::dot(ab_perp_on_tri_plane, a0) > 0  {
            // -- excludes ABC, since this is effectively a plane side test for one plane of the
            // -- triangles's corresponding prism

            // -- thus, the result must be AB
            let ab_perp_to_origin = glm::cross(glm::cross(ab, ao), ab);
            update_simplex_result2(self.a, self.b, ab_perp_to_origin)
        }
        // -- AB, AC excluded beyond here
        else {
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
}

impl S4Simplex {
    pub fn update_simplex(&self) -> (ESimplex, Vec3) {
        // -- possible voronoi regions:
        // -- A, B, C, D, AB, AC, AD, BC, BD, CD, ABC, ABD, ACD, BCD
        // -- A is always the furthest point in the direction perp to ABC, in the direction of O
        // -- Again, we can exclude any not including A, and A itself, leaving:
        // -- AB, AC, AD, ABC, ABD, ACD

        // -- verifying triangle winding
        break_assert!(glm::dot(self.a - self.b, glm::cross(self.c - self.b, self.d - self.b)) > 0);

        // -- we will take an easier approach, and try and figure out which triangle (if any) is
        // -- closest to origin, then fall back to S3Simplex::update_simplex

        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let ad = self.d - self.a;
        let ao = -a;

        let abc_perp = glm::cross(ab, ac);
        let abd_perp = glm::cross(ad, ab);
        let acd_perp = glm::cross(ac, ad);

        if glm::dot(abc_perp, a0) > 0 {
            // -- could be closest to ABC, ABD or ACD, but not inside

        }
        else if glm::dot(abd_perp, a0) > 0 {
            // -- could be closest to ABD or ACD

        }
        else if glm::dot(acd_perp, a0) > 0 {
            // -- must be closest to ACD
            let acd_simplex = S3Simplex{
                a: self.a,
                b: self.c,
                c: self.d,
            };
            return acd_simplex.update_simplex();
        }
        else {
            // -- intersection, we are not outside any of the planes
            let bcd_perp = glm::cross(self.c - self.b, self.d - self.b);
            break_assert!(glm::dot(bcd_perp, a0) <= 0);
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