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
    pub textures: Vec<Texture>,
    #[allow(dead_code)]
    pub lights: Vec<PointLight>,
    pub skybox: Skybox,
}

pub fn temple_scene() -> io::Result<Scene> {
    let textures = vec![
        Texture::from_ppm("assets/textures/piedra.ppm")?,
        Texture::from_ppm("assets/textures/musgo.ppm")?,
        Texture::from_ppm("assets/textures/madera.ppm")?,
        Texture::from_ppm("assets/textures/oro.ppm")?,
    ];

    let mut stone = Material::new(LinearColor::new(0.92, 0.94, 0.90));
    stone.texture_id = Some(0);
    stone.specular = 0.15;
    stone.shininess = 24.0;
    stone.reflectivity = 0.05;

    let mut moss = Material::new(LinearColor::new(0.82, 0.94, 0.78));
    moss.texture_id = Some(1);
    moss.specular = 0.04;
    moss.shininess = 8.0;

    let mut wood = Material::new(LinearColor::new(0.92, 0.82, 0.68));
    wood.texture_id = Some(2);
    wood.specular = 0.18;
    wood.shininess = 20.0;
    wood.reflectivity = 0.04;

    let mut gold = Material::new(LinearColor::new(1.0, 0.90, 0.55));
    gold.texture_id = Some(3);
    gold.specular = 0.85;
    gold.shininess = 96.0;
    gold.reflectivity = 0.65;

    let materials = vec![stone, moss, wood, gold];

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

    Ok(Scene {
        objects,
        materials,
        textures,
        lights: Vec::new(),
        skybox: Skybox {
            color: Color::new(20, 27, 38),
        },
    })
}
use std::io;
