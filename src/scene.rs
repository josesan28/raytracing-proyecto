use crate::color::{Color, LinearColor};
use crate::cube::Cube;
use crate::material::{Material, RayIntersect};
use crate::math::Vec3;
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

pub fn temple_scene() -> Scene {
    let materials = vec![
        Material::new(LinearColor::from(Color::new(112, 119, 104))),
        Material::new(LinearColor::from(Color::new(82, 104, 76))),
        Material::new(LinearColor::from(Color::new(91, 76, 58))),
        Material::new(LinearColor::from(Color::new(190, 143, 48))),
    ];

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        // Plataforma y camino central.
        Box::new(Cube::new(
            Vec3::new(-5.0, -1.0, -10.0),
            Vec3::new(5.0, -0.7, -1.0),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.9, -0.7, -3.0),
            Vec3::new(0.9, -0.5, -1.2),
            2,
        )),
        Box::new(Cube::new(
            Vec3::new(-1.0, -0.7, -5.0),
            Vec3::new(1.0, -0.45, -3.1),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-1.1, -0.7, -7.0),
            Vec3::new(1.1, -0.4, -5.1),
            2,
        )),
        // Columnas y dintel del arco principal.
        Box::new(Cube::new(
            Vec3::new(-2.5, -0.7, -7.2),
            Vec3::new(-1.5, 2.8, -6.2),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(1.5, -0.7, -7.2),
            Vec3::new(2.5, 2.8, -6.2),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-2.7, 2.8, -7.25),
            Vec3::new(2.7, 3.6, -6.15),
            0,
        )),
        // Bloques superiores dan una silueta escalonada al arco.
        Box::new(Cube::new(
            Vec3::new(-2.1, 3.6, -7.2),
            Vec3::new(-0.7, 4.1, -6.2),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(0.7, 3.6, -7.2),
            Vec3::new(2.1, 4.1, -6.2),
            1,
        )),
        // Altar visible a través del arco.
        Box::new(Cube::new(
            Vec3::new(-1.1, -0.7, -8.6),
            Vec3::new(1.1, 0.0, -7.4),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.65, 0.0, -8.35),
            Vec3::new(0.65, 0.8, -7.65),
            3,
        )),
        // Muros rotos laterales de diferentes alturas.
        Box::new(Cube::new(
            Vec3::new(-4.8, -0.7, -8.8),
            Vec3::new(-3.8, 1.8, -4.0),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-3.8, -0.7, -8.8),
            Vec3::new(-2.9, 1.0, -5.0),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(3.7, -0.7, -9.0),
            Vec3::new(4.8, 2.1, -4.8),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(2.9, -0.7, -8.2),
            Vec3::new(3.7, 1.2, -5.4),
            0,
        )),
        // Bloques caídos que rompen la simetría del camino.
        Box::new(Cube::new(
            Vec3::new(-2.8, -0.7, -3.8),
            Vec3::new(-1.8, -0.05, -2.8),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(2.0, -0.7, -4.6),
            Vec3::new(3.1, 0.15, -3.6),
            1,
        )),
    ];

    Scene {
        objects,
        materials,
        textures: Vec::new(),
        lights: Vec::new(),
        skybox: Skybox {
            color: Color::new(20, 27, 38),
        },
    }
}
