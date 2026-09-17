use crate::color::Color;
use crate::math::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub diffuse: Color,
}

impl Material {
    pub const fn new(diffuse: Color) -> Self {
        Self { diffuse }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Intersect {
    #[allow(dead_code)]
    pub point: Vec3,
    #[allow(dead_code)]
    pub normal: Vec3,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material: Material,
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Material) -> Self {
        Self {
            point,
            normal,
            distance,
            is_intersecting: true,
            material,
        }
    }

    pub fn empty() -> Self {
        Self {
            point: Vec3::default(),
            normal: Vec3::default(),
            distance: 0.0,
            is_intersecting: false,
            material: Material::new(Color::new(0, 0, 0)),
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
