// port of math.h from previous projects

use std::f32;
use std::f64;
use std::ops::Neg;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;

// -- $$$FRK(TODO): consider moving format impls to a different file
use std::fmt;

// -- $$$FKR(Note): both f32 and f64 have to_radians() functions, so this is wholly unecessary,
// -- but I did it as an exercise
pub trait HasPi {
    fn pi() -> Self;
}

impl HasPi for f32 {
    fn pi() -> f32 {
        return f32::consts::PI;
    }
}

impl HasPi for f64 {
    fn pi() -> f64 {
        return f64::consts::PI;
    }
}

#[allow(dead_code)]
pub fn degtorad<T: HasPi + From<f32>>(deg : T) -> T 
    where T: Div<Output = T>
{
    let denom : T = T::from(180.0);
    let pi : T = T::pi();
    let denomdivpi : T = denom / pi;
    let result = deg / denomdivpi; 
    return result;
}

// -- SVec3f ----------------------------------------------------------------------

#[allow(dead_code)]
pub struct SVec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[allow(dead_code)]
impl SVec3f {
    pub fn uniform(x: f32) -> Self {
        SVec3f{x: x, y: x, z: x}
    }

    pub fn unitx() -> Self {
        SVec3f{x: 1.0, y: 0.0, z: 0.0}
    }
    pub fn unity() -> Self {
        SVec3f{x: 1.0, y: 0.0, z: 0.0}
    }
    pub fn unitz() -> Self {
        SVec3f{x: 1.0, y: 0.0, z: 0.0}
    }
}

impl Default for SVec3f {
    fn default() -> SVec3f {
        SVec3f{x: 0.0, y: 0.0, z: 0.0}
    }
}

#[allow(dead_code)]
impl Neg for SVec3f {
    type Output = SVec3f;
    fn neg(self) -> SVec3f {
        SVec3f{x: -self.x, y: -self.y, z:-self.z}
    }
}

#[allow(dead_code)]
impl Add for SVec3f {
    type Output = SVec3f;
    fn add(self, other: SVec3f) -> SVec3f {
        SVec3f{x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

#[allow(dead_code)]
impl Mul<SVec3f> for f32 {
    type Output = SVec3f;
    fn mul(self, other: SVec3f) -> SVec3f {
        SVec3f{x: self * other.x, y: self * other.y, z: self * other.z}
    }
}

impl fmt::Display for SVec3f {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
