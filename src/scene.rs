use std::io;

use crate::color::LinearColor;
use crate::cube::Cube;
use crate::material::{Material, RayIntersect};
use crate::math::Vec3;
use crate::texture::Texture;

pub struct PointLight {
    pub position: Vec3,
    pub color: LinearColor,
    pub intensity: f32,
}

pub struct Skybox {
    pub texture_id: usize,
}

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub lights: Vec<PointLight>,
    pub skybox: Skybox,
}

pub fn temple_scene() -> io::Result<Scene> {
    let textures = vec![
        Texture::from_ppm("assets/textures/piedra.ppm")?,
        Texture::from_ppm("assets/textures/musgo.ppm")?,
        Texture::from_ppm("assets/textures/madera.ppm")?,
        Texture::from_ppm("assets/textures/oro.ppm")?,
        Texture::from_ppm("assets/textures/piedra_normal.ppm")?,
        Texture::from_ppm("assets/textures/agua.ppm")?,
        Texture::from_ppm("assets/textures/agua_normal.ppm")?,
        Texture::from_ppm("assets/textures/emisivo.ppm")?,
        Texture::from_ppm("assets/textures/skybox_atardecer.ppm")?,
    ];

    let mut stone = Material::new(LinearColor::new(0.92, 0.94, 0.90));
    stone.texture_id = Some(0);
    stone.texture_scale = crate::math::Vec2::new(2.0, 2.0);
    stone.normal_map_id = Some(4);
    stone.specular = 0.15;
    stone.shininess = 24.0;
    stone.reflectivity = 0.05;

    let mut moss = Material::new(LinearColor::new(0.82, 0.94, 0.78));
    moss.texture_id = Some(1);
    moss.texture_scale = crate::math::Vec2::new(3.0, 3.0);
    moss.specular = 0.04;
    moss.shininess = 8.0;

    let mut wood = Material::new(LinearColor::new(0.92, 0.82, 0.68));
    wood.texture_id = Some(2);
    wood.texture_scale = crate::math::Vec2::new(2.0, 2.0);
    wood.specular = 0.18;
    wood.shininess = 20.0;
    wood.reflectivity = 0.04;

    let mut gold = Material::new(LinearColor::new(1.0, 0.90, 0.55));
    gold.texture_id = Some(3);
    gold.specular = 0.85;
    gold.shininess = 96.0;
    gold.reflectivity = 0.65;

    let mut water = Material::new(LinearColor::new(0.58, 0.86, 0.92));
    water.texture_id = Some(5);
    water.texture_scale = crate::math::Vec2::new(2.0, 2.0);
    water.normal_map_id = Some(6);
    water.specular = 0.9;
    water.shininess = 128.0;
    water.transparency = 0.78;
    water.reflectivity = 0.12;
    water.refractive_index = 1.33;

    let mut emissive = Material::new(LinearColor::new(1.0, 0.52, 0.12));
    emissive.texture_id = Some(7);
    emissive.specular = 0.25;
    emissive.shininess = 32.0;
    emissive.emission = LinearColor::new(1.8, 0.72, 0.10);

    let materials = vec![stone, moss, wood, gold, water, emissive];

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        // Plataforma y camino central.
        Box::new(Cube::new(
            Vec3::new(-6.0, -1.0, -10.0),
            Vec3::new(6.0, -0.7, 4.5),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.9, -0.7, -3.0),
            Vec3::new(0.9, -0.5, -1.2),
            2,
        )),
        Box::new(Cube::new(
            Vec3::new(-1.0, -0.7, -5.0),
            Vec3::new(1.0, -0.65, -3.1),
            0,
        )),
        // Fuente con agua volumétrica y mosaicos sumergidos.
        Box::new(Cube::new(
            Vec3::new(-0.82, -0.63, -4.85),
            Vec3::new(0.82, -0.14, -3.55),
            4,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.70, -0.62, -4.65),
            Vec3::new(-0.25, -0.52, -3.75),
            3,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.20, -0.62, -4.65),
            Vec3::new(0.20, -0.52, -3.75),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(0.25, -0.62, -4.65),
            Vec3::new(0.70, -0.52, -3.75),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-1.15, -0.65, -5.15),
            Vec3::new(-0.85, -0.03, -3.25),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(0.85, -0.65, -5.15),
            Vec3::new(1.15, -0.03, -3.25),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.85, -0.65, -5.15),
            Vec3::new(0.85, -0.03, -4.85),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.85, -0.65, -3.55),
            Vec3::new(0.85, -0.03, -3.25),
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
        // Basas y capiteles escalonados rompen la forma monolítica.
        Box::new(Cube::new(
            Vec3::new(-2.72, -0.7, -7.38),
            Vec3::new(-1.28, -0.34, -6.02),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(1.28, -0.7, -7.38),
            Vec3::new(2.72, -0.34, -6.02),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-2.72, 2.45, -7.38),
            Vec3::new(-1.28, 2.88, -6.02),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(1.28, 2.45, -7.38),
            Vec3::new(2.72, 2.88, -6.02),
            0,
        )),
        // Placas de musgo adheridas a las columnas.
        Box::new(Cube::new(
            Vec3::new(-2.42, 0.35, -6.19),
            Vec3::new(-2.08, 2.15, -6.07),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(1.88, 1.25, -6.19),
            Vec3::new(2.35, 2.58, -6.07),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-2.7, 2.8, -7.25),
            Vec3::new(2.7, 3.6, -6.15),
            0,
        )),
        // Antorchas emisivas con soportes de madera.
        Box::new(Cube::new(
            Vec3::new(-1.42, 0.25, -6.12),
            Vec3::new(-1.18, 1.18, -5.88),
            2,
        )),
        Box::new(Cube::new(
            Vec3::new(1.18, 0.25, -6.12),
            Vec3::new(1.42, 1.18, -5.88),
            2,
        )),
        Box::new(Cube::new(
            Vec3::new(-1.48, 1.15, -6.15),
            Vec3::new(-1.12, 1.72, -5.80),
            5,
        )),
        Box::new(Cube::new(
            Vec3::new(1.12, 1.15, -6.15),
            Vec3::new(1.48, 1.72, -5.80),
            5,
        )),
        // Runas de cubos delgados sobre la cara frontal del dintel.
        Box::new(Cube::new(
            Vec3::new(-1.05, 3.00, -6.14),
            Vec3::new(-0.87, 3.43, -6.02),
            5,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.87, 3.00, -6.14),
            Vec3::new(-0.52, 3.15, -6.02),
            5,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.10, 3.08, -6.14),
            Vec3::new(0.10, 3.50, -6.02),
            5,
        )),
        Box::new(Cube::new(
            Vec3::new(0.52, 3.28, -6.14),
            Vec3::new(1.05, 3.43, -6.02),
            5,
        )),
        Box::new(Cube::new(
            Vec3::new(0.87, 3.00, -6.14),
            Vec3::new(1.05, 3.43, -6.02),
            5,
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
        Box::new(Cube::new(
            Vec3::new(-0.43, 0.8, -8.20),
            Vec3::new(0.43, 1.12, -7.80),
            3,
        )),
        Box::new(Cube::new(
            Vec3::new(-0.18, 1.12, -8.08),
            Vec3::new(0.18, 1.48, -7.92),
            5,
        )),
        // Escalones y santuario posterior enmarcan el altar sin cerrar el cielo.
        Box::new(Cube::new(
            Vec3::new(-1.55, -0.7, -9.15),
            Vec3::new(1.55, -0.48, -8.55),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-1.28, -0.48, -9.05),
            Vec3::new(1.28, -0.22, -8.58),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-3.45, -0.7, -9.65),
            Vec3::new(-1.45, 2.15, -9.18),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(1.45, -0.7, -9.65),
            Vec3::new(3.45, 1.65, -9.18),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-3.30, 1.72, -9.17),
            Vec3::new(-2.25, 2.48, -9.05),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(2.35, 1.20, -9.17),
            Vec3::new(3.30, 2.02, -9.05),
            1,
        )),
        // Pedestales secundarios sugieren estatuas perdidas a ambos lados.
        Box::new(Cube::new(
            Vec3::new(-3.20, -0.7, -8.95),
            Vec3::new(-2.35, -0.12, -8.18),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(2.35, -0.7, -8.95),
            Vec3::new(3.20, -0.12, -8.18),
            0,
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
        // Vegetación en masas asimétricas alrededor de las ruinas.
        Box::new(Cube::new(
            Vec3::new(-4.15, -0.7, -4.15),
            Vec3::new(-3.25, 0.15, -3.35),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-4.55, -0.7, -5.05),
            Vec3::new(-3.80, 0.65, -4.35),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(3.35, -0.7, -4.65),
            Vec3::new(4.35, 0.35, -3.70),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(4.05, -0.7, -6.15),
            Vec3::new(4.75, 0.85, -5.35),
            1,
        )),
        // Troncos y copas cúbicas crean profundidad detrás del templo.
        Box::new(Cube::new(
            Vec3::new(-5.35, -0.7, -8.45),
            Vec3::new(-4.95, 2.65, -8.05),
            2,
        )),
        Box::new(Cube::new(
            Vec3::new(-5.80, 2.05, -8.85),
            Vec3::new(-4.45, 3.55, -7.55),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(4.85, -0.7, -9.10),
            Vec3::new(5.20, 2.25, -8.75),
            2,
        )),
        Box::new(Cube::new(
            Vec3::new(4.35, 1.75, -9.50),
            Vec3::new(5.75, 3.20, -8.15),
            1,
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
        // Una columna derrumbada conduce la vista desde el primer plano.
        Box::new(Cube::new(
            Vec3::new(-4.45, -0.7, -2.35),
            Vec3::new(-3.65, -0.18, -1.55),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-3.58, -0.7, -2.28),
            Vec3::new(-2.82, -0.12, -1.60),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-2.74, -0.7, -2.20),
            Vec3::new(-2.05, -0.22, -1.65),
            1,
        )),
    ];

    Ok(Scene {
        objects,
        materials,
        textures,
        lights: vec![
            PointLight {
                position: Vec3::new(-1.30, 1.44, -5.65),
                color: LinearColor::new(1.0, 0.48, 0.18),
                intensity: 10.0,
            },
            PointLight {
                position: Vec3::new(1.30, 1.44, -5.65),
                color: LinearColor::new(1.0, 0.58, 0.24),
                intensity: 10.0,
            },
            PointLight {
                position: Vec3::new(0.0, 5.5, -2.0),
                color: LinearColor::new(0.45, 0.58, 0.82),
                intensity: 14.0,
            },
            PointLight {
                position: Vec3::new(-5.5, 7.5, 1.5),
                color: LinearColor::new(1.0, 0.66, 0.38),
                intensity: 38.0,
            },
        ],
        skybox: Skybox { texture_id: 8 },
    })
}
