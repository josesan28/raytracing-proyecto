use crate::math::Vec3;

pub struct BoundsRay {
    origin: Vec3,
    inverse: Vec3,
}

impl BoundsRay {
    pub fn new(origin: &Vec3, direction: &Vec3) -> Self {
        let reciprocal = |value: f32| {
            if value.abs() <= f32::EPSILON {
                0.0
            } else {
                1.0 / value
            }
        };
        Self {
            origin: *origin,
            inverse: Vec3::new(
                reciprocal(direction.x),
                reciprocal(direction.y),
                reciprocal(direction.z),
            ),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn union(self, other: Self) -> Self {
        Self::new(
            Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }

    pub fn centroid(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extent(self) -> Vec3 {
        self.max - self.min
    }

    pub fn is_hit(self, origin: &Vec3, direction: &Vec3, max_distance: f32) -> bool {
        self.is_hit_cached(&BoundsRay::new(origin, direction), max_distance)
    }

    // La BVH reutiliza las mismas tres inversas en todos los nodos de un rayo.
    pub fn is_hit_cached(self, ray: &BoundsRay, max_distance: f32) -> bool {
        let mut near = 0.0_f32;
        let mut far = max_distance;
        let axes = [
            (ray.origin.x, ray.inverse.x, self.min.x, self.max.x),
            (ray.origin.y, ray.inverse.y, self.min.y, self.max.y),
            (ray.origin.z, ray.inverse.z, self.min.z, self.max.z),
        ];

        for (axis_origin, inverse, min, max) in axes {
            if inverse == 0.0 {
                if axis_origin < min || axis_origin > max {
                    return false;
                }
                continue;
            }

            let mut axis_near = (min - axis_origin) * inverse;
            let mut axis_far = (max - axis_origin) * inverse;
            if axis_near > axis_far {
                std::mem::swap(&mut axis_near, &mut axis_far);
            }
            near = near.max(axis_near);
            far = far.min(axis_far);
            if near > far {
                return false;
            }
        }

        far >= 0.0
    }
}
