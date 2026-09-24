use std::f32::consts::PI;

use crate::material::{Intersect, MaterialId, RayIntersect};
use crate::math::{Vec2, Vec3};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_id: MaterialId,
}

impl Sphere {
    pub const fn new(center: Vec3, radius: f32, material_id: MaterialId) -> Self {
        Self {
            center,
            radius,
            material_id,
        }
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let oc = *ray_origin - self.center;
        let a = ray_direction.dot(*ray_direction);
        let b = 2.0 * oc.dot(*ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return Intersect::empty();
        }

        let square_root = discriminant.sqrt();
        let nearest = (-b - square_root) / (2.0 * a);
        let farthest = (-b + square_root) / (2.0 * a);
        let distance = if nearest > 0.001 {
            nearest
        } else if farthest > 0.001 {
            farthest
        } else {
            return Intersect::empty();
        };

        let point = *ray_origin + *ray_direction * distance;
        let outward_normal = (point - self.center).normalize();
        let uv = Vec2::new(
            0.5 + outward_normal.z.atan2(outward_normal.x) / (2.0 * PI),
            0.5 - outward_normal.y.asin() / PI,
        );

        Intersect::new(
            point,
            outward_normal,
            distance,
            uv,
            self.material_id,
            ray_direction,
        )
    }
}
