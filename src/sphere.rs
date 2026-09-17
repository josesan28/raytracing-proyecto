use crate::material::{Intersect, Material, RayIntersect};
use crate::math::Vec3;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub const fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
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
        let normal = (point - self.center).normalize();

        Intersect::new(point, normal, distance, self.material)
    }
}
