use crate::math::Vec3;

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
        let mut near = 0.0_f32;
        let mut far = max_distance;
        let axes = [
            (origin.x, direction.x, self.min.x, self.max.x),
            (origin.y, direction.y, self.min.y, self.max.y),
            (origin.z, direction.z, self.min.z, self.max.z),
        ];

        for (axis_origin, axis_direction, min, max) in axes {
            if axis_direction.abs() <= f32::EPSILON {
                if axis_origin < min || axis_origin > max {
                    return false;
                }
                continue;
            }

            let inverse = 1.0 / axis_direction;
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
