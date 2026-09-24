use crate::color::{Color, LinearColor};
use crate::material::{Material, RayIntersect};
use crate::math::Vec3;
use crate::sphere::Sphere;
use crate::texture::Texture;

#[allow(dead_code)]
pub struct PointLight {
    pub position: Vec3,
    pub color: LinearColor,
    pub intensity: f32,
}

pub struct Skybox {
    pub color: Color,
}

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
    pub materials: Vec<Material>,
    #[allow(dead_code)]
    pub textures: Vec<Texture>,
    #[allow(dead_code)]
    pub lights: Vec<PointLight>,
    pub skybox: Skybox,
}

pub fn zbuffer_scene() -> Scene {
    let materials = vec![
        Material::new(LinearColor::from(Color::new(245, 245, 235))),
        Material::new(LinearColor::from(Color::new(220, 45, 55))),
        Material::new(LinearColor::from(Color::new(55, 180, 100))),
    ];

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        // La esfera cercana aparece primero en el arreglo a propósito.
        Box::new(Sphere::new(Vec3::new(0.0, 0.0, -3.0), 1.0, 0)),
        // Esta esfera está más lejos y se superpone con la anterior.
        Box::new(Sphere::new(Vec3::new(0.0, 0.0, -4.0), 1.0, 1)),
        Box::new(Sphere::new(Vec3::new(1.35, -0.35, -4.2), 0.55, 2)),
    ];

    Scene {
        objects,
        materials,
        textures: Vec::new(),
        lights: Vec::new(),
        skybox: Skybox {
            color: Color::new(8, 10, 18),
        },
    }
}
