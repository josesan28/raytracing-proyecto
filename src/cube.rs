use crate::material::{Intersect, MaterialId, RAY_EPSILON, RayIntersect};
use crate::math::{Vec2, Vec3};

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material_id: MaterialId,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material_id: MaterialId) -> Self {
        assert!(min.x < max.x && min.y < max.y && min.z < max.z);
        Self {
            min,
            max,
            material_id,
        }
    }

    fn surface_coordinates(&self, point: Vec3, normal: Vec3) -> (Vec2, Vec3, Vec3) {
        let size = self.max - self.min;

        // Cada cara se observa desde fuera. U avanza hacia la derecha y V hacia arriba.
        if normal.x > 0.5 {
            (
                Vec2::new(
                    (self.max.z - point.z) / size.z,
                    (point.y - self.min.y) / size.y,
                ),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        } else if normal.x < -0.5 {
            (
                Vec2::new(
                    (point.z - self.min.z) / size.z,
                    (point.y - self.min.y) / size.y,
                ),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        } else if normal.y > 0.5 {
            (
                Vec2::new(
                    (point.x - self.min.x) / size.x,
                    (self.max.z - point.z) / size.z,
                ),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
            )
        } else if normal.y < -0.5 {
            (
                Vec2::new(
                    (point.x - self.min.x) / size.x,
                    (point.z - self.min.z) / size.z,
                ),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            )
        } else if normal.z > 0.5 {
            (
                Vec2::new(
                    (point.x - self.min.x) / size.x,
                    (point.y - self.min.y) / size.y,
                ),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        } else {
            (
                Vec2::new(
                    (self.max.x - point.x) / size.x,
                    (point.y - self.min.y) / size.y,
                ),
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut near_distance = f32::NEG_INFINITY;
        let mut far_distance = f32::INFINITY;
        let mut near_normal = Vec3::default();
        let mut far_normal = Vec3::default();

        let slabs = [
            (
                ray_origin.x,
                ray_direction.x,
                self.min.x,
                self.max.x,
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            ),
            (
                ray_origin.y,
                ray_direction.y,
                self.min.y,
                self.max.y,
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            (
                ray_origin.z,
                ray_direction.z,
                self.min.z,
                self.max.z,
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
        ];

        for (origin, direction, min, max, min_normal, max_normal) in slabs {
            if direction.abs() <= f32::EPSILON {
                if origin < min || origin > max {
                    return Intersect::empty();
                }
                continue;
            }

            let inverse_direction = 1.0 / direction;
            let mut axis_near = (min - origin) * inverse_direction;
            let mut axis_far = (max - origin) * inverse_direction;
            let mut axis_near_normal = min_normal;
            let mut axis_far_normal = max_normal;

            if axis_near > axis_far {
                std::mem::swap(&mut axis_near, &mut axis_far);
                std::mem::swap(&mut axis_near_normal, &mut axis_far_normal);
            }

            if axis_near > near_distance {
                near_distance = axis_near;
                near_normal = axis_near_normal;
            }
            if axis_far < far_distance {
                far_distance = axis_far;
                far_normal = axis_far_normal;
            }

            if near_distance > far_distance {
                return Intersect::empty();
            }
        }

        let (distance, outward_normal) = if near_distance > RAY_EPSILON {
            (near_distance, near_normal)
        } else if far_distance > RAY_EPSILON {
            (far_distance, far_normal)
        } else {
            return Intersect::empty();
        };

        let point = *ray_origin + *ray_direction * distance;
        let (uv, tangent, bitangent) = self.surface_coordinates(point, outward_normal);

        Intersect::new(
            point,
            outward_normal,
            tangent,
            bitangent,
            distance,
            uv,
            self.material_id,
            ray_direction,
        )
    }

    fn bounding_box(&self) -> Aabb {
        Aabb::new(self.min, self.max)
    }

    fn material_id(&self) -> MaterialId {
        self.material_id
    }

    fn intersects_before(
        &self,
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        max_distance: f32,
    ) -> bool {
        self.bounding_box()
            .is_hit(ray_origin, ray_direction, max_distance)
    }
}
use crate::bounding_box::Aabb;
