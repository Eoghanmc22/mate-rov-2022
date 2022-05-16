use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, Default)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Vec3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn uniform(v: f64) -> Self {
        Self::new(v, v, v)
    }

    pub fn squared_length(&self) -> f64 {
        self.x * self.x +  self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        self.squared_length().sqrt()
    }

    pub fn norm(&self) -> Self {
        let length = self.length();
        Self::new(self.x / length, self.y / length, self.z / length)
    }
}

impl Add for Vec3d {
    type Output = Vec3d;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3d::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z
        )
    }
}

impl AddAssign for Vec3d {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for Vec3d {
    type Output = Vec3d;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3d::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z
        )
    }
}

impl SubAssign for Vec3d {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<f64> for Vec3d {
    type Output = Vec3d;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3d::new(
            self.x * rhs,
            self.y * rhs,
            self.z * rhs
        )
    }
}

impl MulAssign<f64> for Vec3d {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs
    }
}