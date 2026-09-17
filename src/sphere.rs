use crate::color::Color;
use crate::math::Vec3;

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<f32>;
}

pub struct Sphere {
    center: Vec3,
    radius: f32,
    color: Color,
}

impl Sphere {
    pub const fn new(center: Vec3, radius: f32, color: Color) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }

    pub const fn color(&self) -> Color {
        self.color
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<f32> {
        let oc = ray_origin - self.center;
        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * oc.dot(ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let square_root = discriminant.sqrt();
        let nearest = (-b - square_root) / (2.0 * a);
        let farthest = (-b + square_root) / (2.0 * a);

        if nearest > 0.001 {
            Some(nearest)
        } else if farthest > 0.001 {
            Some(farthest)
        } else {
            None
        }
    }
}
