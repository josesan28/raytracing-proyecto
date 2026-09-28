use crate::bounding_box::Aabb;
use crate::color::LinearColor;
use crate::math::{Vec2, Vec3};

pub type MaterialId = usize;
pub type TextureId = usize;
pub const RAY_EPSILON: f32 = 0.001;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: LinearColor,
    pub texture_scale: Vec2,
    pub texture_id: Option<TextureId>,
    pub normal_map_id: Option<TextureId>,
    pub specular: f32,
    pub shininess: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refractive_index: f32,
    pub emission: LinearColor,
}

impl Material {
    pub const fn new(albedo: LinearColor) -> Self {
        Self {
            albedo,
            texture_scale: Vec2::new(1.0, 1.0),
            texture_id: None,
            normal_map_id: None,
            specular: 0.0,
            shininess: 1.0,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emission: LinearColor::new(0.0, 0.0, 0.0),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub geometric_normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv: Vec2,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material_id: MaterialId,
    pub front_face: bool,
}

impl Intersect {
    pub fn new(
        point: Vec3,
        outward_normal: Vec3,
        tangent: Vec3,
        outward_bitangent: Vec3,
        distance: f32,
        uv: Vec2,
        material_id: MaterialId,
        ray_direction: &Vec3,
    ) -> Self {
        let front_face = ray_direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            outward_normal * -1.0
        };
        let bitangent = if front_face {
            outward_bitangent
        } else {
            outward_bitangent * -1.0
        };

        Self {
            point,
            normal,
            geometric_normal: outward_normal,
            tangent,
            bitangent,
            uv,
            distance,
            is_intersecting: true,
            material_id,
            front_face,
        }
    }

    pub fn empty() -> Self {
        Self {
            point: Vec3::default(),
            normal: Vec3::default(),
            geometric_normal: Vec3::default(),
            tangent: Vec3::default(),
            bitangent: Vec3::default(),
            uv: Vec2::default(),
            distance: 0.0,
            is_intersecting: false,
            material_id: 0,
            front_face: false,
        }
    }
}

pub trait RayIntersect: Sync {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
    fn bounding_box(&self) -> Aabb;
}
