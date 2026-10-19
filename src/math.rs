use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::NoUninit)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    pub const fn dot(self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn normalize(self) -> Vec2 {
        self / self.length()
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;

    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::NoUninit)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    pub const fn dot(self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn normalize(self) -> Vec3 {
        self / self.length()
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<Vec3> for f32 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self / rhs.x,
            y: self / rhs.y,
            z: self / rhs.z,
        }
    }
}

impl From<Vec2> for Vec3 {
    fn from(value: Vec2) -> Self {
        Vec3::new(value.x, value.y, 0.)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::NoUninit)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const fn new(m: [f32; 16]) -> Self {
        Self { m }
    }

    #[rustfmt::skip]
    pub const fn identity() -> Self {
        Self::new([
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ])
    }

    #[rustfmt::skip]
    pub const fn translate(t: Vec3) -> Self {
        Self::new([
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            t.x, t.y, t.z, 1.,
        ])
    }

    #[rustfmt::skip]
    pub const fn scale(s: Vec3) -> Self {
        Self::new([
            s.x, 0., 0., 0.,
            0., s.y, 0., 0.,
            0., 0., s.z, 0.,
            0., 0., 0., 1.,
        ])
    }

    #[rustfmt::skip]
    pub fn rotateX(r: f32) -> Self {
        Self::new([
            1., 0., 0., 0.,
            0., r.cos(), -r.sin(), 0.,
            0., r.sin(), r.cos(), 0.,
            0., 0., 0., 1.,
        ])
    }

    #[rustfmt::skip]
    pub fn rotateY(r: f32) -> Self {
        Self::new([
            r.cos(), 0., r.sin(), 0.,
            0., 1., 0., 0.,
            -r.sin(), 0., r.cos(), 0.,
            0., 0., 0., 1.,
        ])
    }

    #[rustfmt::skip]
    pub fn rotateZ(r: f32) -> Self {
        Self::new([
            r.cos(), -r.sin(), 0., 0.,
            r.sin(), r.cos(), 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ])
    }
}

/*

               0  4  8  12
               1  5  9  13
               2  6  10 14
               3  7  11 15

0  4  8  12    0  4  8  12
1  5  9  13    1  5  9  13
2  6  10 14    2  6  10 14
3  7  11 15    3  7  11 15
*/

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, r: Self) -> Self::Output {
        Self::new([
            // Column 1
            self.m[0] * r.m[0] + self.m[4] * r.m[1] + self.m[8] * r.m[2] + self.m[12] * r.m[3],
            self.m[1] * r.m[0] + self.m[5] * r.m[1] + self.m[9] * r.m[2] + self.m[13] * r.m[3],
            self.m[2] * r.m[0] + self.m[6] * r.m[1] + self.m[10] * r.m[2] + self.m[14] * r.m[3],
            self.m[3] * r.m[0] + self.m[7] * r.m[1] + self.m[11] * r.m[2] + self.m[15] * r.m[3],
            // Column 2
            self.m[0] * r.m[4] + self.m[4] * r.m[5] + self.m[8] * r.m[6] + self.m[12] * r.m[7],
            self.m[1] * r.m[4] + self.m[5] * r.m[5] + self.m[9] * r.m[6] + self.m[13] * r.m[7],
            self.m[2] * r.m[4] + self.m[6] * r.m[5] + self.m[10] * r.m[6] + self.m[14] * r.m[7],
            self.m[3] * r.m[4] + self.m[7] * r.m[5] + self.m[11] * r.m[6] + self.m[15] * r.m[7],
            // Column 3
            self.m[0] * r.m[8] + self.m[4] * r.m[9] + self.m[8] * r.m[10] + self.m[12] * r.m[11],
            self.m[1] * r.m[8] + self.m[5] * r.m[9] + self.m[9] * r.m[10] + self.m[13] * r.m[11],
            self.m[2] * r.m[8] + self.m[6] * r.m[9] + self.m[10] * r.m[10] + self.m[14] * r.m[11],
            self.m[3] * r.m[8] + self.m[7] * r.m[9] + self.m[11] * r.m[10] + self.m[15] * r.m[11],
            // Column 4
            self.m[0] * r.m[12] + self.m[4] * r.m[13] + self.m[8] * r.m[14] + self.m[12] * r.m[15],
            self.m[1] * r.m[12] + self.m[5] * r.m[13] + self.m[9] * r.m[14] + self.m[13] * r.m[15],
            self.m[2] * r.m[12] + self.m[6] * r.m[13] + self.m[10] * r.m[14] + self.m[14] * r.m[15],
            self.m[3] * r.m[12] + self.m[7] * r.m[13] + self.m[11] * r.m[14] + self.m[15] * r.m[15],
        ])
    }
}

impl MulAssign for Mat4 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

/*
               x
               y
               z
               1

0  4  8  12    x'
1  5  9  13    y'
2  6  10 14    z'
3  7  11 15    -
*/

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.m[0] * rhs.x + self.m[4] * rhs.y + self.m[8] * rhs.z + self.m[12],
            self.m[1] * rhs.x + self.m[5] * rhs.y + self.m[9] * rhs.z + self.m[13],
            self.m[2] * rhs.x + self.m[6] * rhs.y + self.m[10] * rhs.z + self.m[14],
        )
    }
}
