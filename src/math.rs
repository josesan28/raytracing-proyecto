use std::ops::{Add, Div, Mul, Sub};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub u: f32,
    pub v: f32,
}

impl Vec2 {
    pub const fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.length();

        if length == 0.0 {
            return self;
        }

        self / length
    }

    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    pub fn refract(self, normal: Self, index_ratio: f32) -> Option<Self> {
        let cosine = (self * -1.0).dot(normal).clamp(0.0, 1.0);
        let discriminant = 1.0 - index_ratio * index_ratio * (1.0 - cosine * cosine);

        if discriminant < 0.0 {
            None
        } else {
            Some(self * index_ratio + normal * (index_ratio * cosine - discriminant.sqrt()))
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}
