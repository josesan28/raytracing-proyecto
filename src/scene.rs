use std::io;

use crate::bvh::Bvh;
use crate::color::LinearColor;
use crate::cube::Cube;
use crate::material::{Material, RayIntersect};
use crate::math::Vec3;
use crate::procedural::{TerrainConfig, append_terrain};
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
    pub bvh: Bvh,
}

pub fn temple_scene() -> io::Result<Scene> {
    let mut textures = vec![
        Texture::from_ppm("assets/textures/piedra.ppm")?,
        Texture::from_ppm("assets/textures/musgo.ppm")?,
        Texture::from_ppm("assets/textures/madera.ppm")?,
        Texture::from_ppm("assets/textures/oro.ppm")?,
        Texture::from_ppm("assets/textures/piedra_normal.ppm")?,
        Texture::from_ppm("assets/textures/agua.ppm")?,
        Texture::from_ppm("assets/textures/agua_normal.ppm")?,
        Texture::from_ppm("assets/textures/emisivo.ppm")?,
        Texture::from_ppm("assets/textures/skybox_atardecer.ppm")?,
        Texture::from_ppm("assets/textures/oro_normal.ppm")?,
        Texture::from_ppm("assets/textures/suelo_selva.ppm")?,
        Texture::from_ppm("assets/textures/suelo_selva_normal.ppm")?,
        Texture::from_ppm("assets/textures/corteza_normal.ppm")?,
        Texture::from_ppm("assets/textures/corteza.ppm")?,
        Texture::from_ppm("assets/textures/hojas_arbol.ppm")?,
        Texture::from_ppm("assets/textures/hojas_arbol_normal.ppm")?,
        Texture::from_ppm("assets/textures/llama_lava.ppm")?,
        Texture::from_ppm("assets/textures/llama_lava_normal.ppm")?,
        Texture::from_ppm("assets/textures/musgo_normal.ppm")?,
        Texture::from_ppm("assets/textures/madera_normal.ppm")?,
    ];

    let mut stone = Material::new(LinearColor::new(0.92, 0.94, 0.90));
    stone.texture_id = Some(0);
    stone.texture_scale = crate::math::Vec2::new(1.0, 1.0);
    stone.normal_map_id = Some(4);
    stone.normal_map_strength = 0.75;
    stone.specular = 0.15;
    stone.shininess = 24.0;

    let mut moss = Material::new(LinearColor::new(0.82, 0.94, 0.78));
    moss.texture_id = Some(1);
    moss.texture_scale = crate::math::Vec2::new(1.75, 1.75);
    moss.normal_map_id = Some(18);
    moss.normal_map_strength = 0.45;
    moss.specular = 0.04;
    moss.shininess = 8.0;

    let mut wood = Material::new(LinearColor::new(0.92, 0.82, 0.68));
    wood.texture_id = Some(2);
    wood.texture_scale = crate::math::Vec2::new(1.5, 1.5);
    wood.normal_map_id = Some(19);
    wood.normal_map_strength = 0.55;
    wood.specular = 0.18;
    wood.shininess = 20.0;

    let mut gold = Material::new(LinearColor::new(1.0, 0.90, 0.55));
    gold.texture_id = Some(3);
    gold.normal_map_id = Some(9);
    gold.normal_map_strength = 0.32;
    gold.specular = 0.85;
    gold.shininess = 96.0;
    gold.reflectivity = 0.65;

    let mut water = Material::new(LinearColor::new(0.58, 0.86, 0.92));
    water.texture_id = Some(5);
    water.texture_scale = crate::math::Vec2::new(1.5, 1.5);
    water.normal_map_id = Some(6);
    water.normal_map_strength = 0.22;
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

    let mut ground = Material::new(LinearColor::new(0.76, 0.82, 0.72));
    ground.texture_id = Some(10);
    ground.normal_map_id = Some(11);
    ground.normal_map_strength = 0.38;
    ground.specular = 0.05;
    ground.shininess = 10.0;

    let mut bark = Material::new(LinearColor::new(0.78, 0.66, 0.50));
    bark.texture_id = Some(13);
    bark.texture_scale = crate::math::Vec2::new(1.0, 2.0);
    bark.normal_map_id = Some(12);
    bark.normal_map_strength = 0.58;
    bark.specular = 0.06;
    bark.shininess = 12.0;

    let mut leaves = Material::new(LinearColor::new(0.42, 0.68, 0.32));
    leaves.texture_id = Some(14);
    leaves.normal_map_id = Some(15);
    leaves.normal_map_strength = 0.35;
    leaves.specular = 0.03;
    leaves.shininess = 8.0;

    let mut flame = Material::new(LinearColor::new(1.0, 0.68, 0.32));
    flame.texture_id = Some(16);
    flame.normal_map_id = Some(17);
    flame.normal_map_strength = 0.20;
    flame.specular = 0.20;
    flame.shininess = 24.0;
    flame.emission = LinearColor::new(2.2, 0.72, 0.10);

    let materials = vec![
        stone, moss, wood, gold, water, emissive, ground, bark, leaves, flame,
    ];

    // Solo albedo opaco: el cielo, el agua y la emisión conservan sus valores.
    for texture_id in [0, 1, 2, 3, 10, 13, 14] {
        textures[texture_id].enhance_detail(0.65);
    }
    // Cada material de esta escena posee su propio mapa normal. Si se comparte
    // uno con distinta intensidad, debe cargarse como una textura independiente.
    for material in &materials {
        if let Some(normal_map_id) = material.normal_map_id {
            textures[normal_map_id].prepare_normal_map(material.normal_map_strength);
        }
    }

    let mut objects: Vec<Box<dyn RayIntersect>> = vec![
        // Plataforma y camino central.
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
        // Fuente y mosaicos bajo el agua.
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
        // Bases y capiteles de las columnas.
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
        // Musgo sobre las columnas.
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
            9,
        )),
        Box::new(Cube::new(
            Vec3::new(1.12, 1.15, -6.15),
            Vec3::new(1.48, 1.72, -5.80),
            9,
        )),
        // Bloques superiores del arco.
        Box::new(Cube::new(
            Vec3::new(-2.1, 3.6, -7.2),
            Vec3::new(-0.7, 4.1, -6.2),
            1,
        )),
        // Altar central.
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
        // Escalones y muros del santuario posterior.
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
        // Pedestales laterales.
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
            Vec3::new(-4.1, 1.9, -6.95),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-4.72, -0.7, -6.78),
            Vec3::new(-3.98, 1.12, -5.38),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-4.6, -0.7, -5.22),
            Vec3::new(-3.9, 0.48, -4.0),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(-3.92, -0.7, -8.65),
            Vec3::new(-3.18, 0.92, -7.18),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(-3.76, -0.7, -7.0),
            Vec3::new(-3.04, 0.38, -5.5),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(4.08, -0.7, -9.0),
            Vec3::new(4.8, 2.12, -7.25),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(3.95, -0.7, -7.08),
            Vec3::new(4.72, 1.42, -5.7),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(3.88, -0.7, -5.54),
            Vec3::new(4.58, 0.68, -4.5),
            1,
        )),
        Box::new(Cube::new(
            Vec3::new(3.15, -0.7, -8.55),
            Vec3::new(3.88, 1.08, -7.12),
            0,
        )),
        Box::new(Cube::new(
            Vec3::new(3.05, -0.7, -6.94),
            Vec3::new(3.74, 0.44, -5.62),
            1,
        )),
        // Vegetación junto a las ruinas.
        Box::new(Cube::new(
            Vec3::new(-4.15, -0.7, -4.15),
            Vec3::new(-3.25, 0.15, -3.35),
            8,
        )),
        Box::new(Cube::new(
            Vec3::new(-4.55, -0.7, -5.05),
            Vec3::new(-3.80, 0.65, -4.35),
            8,
        )),
        Box::new(Cube::new(
            Vec3::new(3.35, -0.7, -4.65),
            Vec3::new(4.35, 0.35, -3.70),
            8,
        )),
        Box::new(Cube::new(
            Vec3::new(4.05, -0.7, -6.15),
            Vec3::new(4.75, 0.85, -5.35),
            8,
        )),
        // Árboles detrás del templo.
        Box::new(Cube::new(
            Vec3::new(-5.35, -0.7, -8.45),
            Vec3::new(-4.95, 2.65, -8.05),
            7,
        )),
        Box::new(Cube::new(
            Vec3::new(-5.80, 2.05, -8.85),
            Vec3::new(-4.45, 3.55, -7.55),
            8,
        )),
        Box::new(Cube::new(
            Vec3::new(4.85, -0.7, -9.10),
            Vec3::new(5.20, 2.25, -8.75),
            7,
        )),
        Box::new(Cube::new(
            Vec3::new(4.35, 1.75, -9.50),
            Vec3::new(5.75, 3.20, -8.15),
            8,
        )),
        // Bloques caídos junto al camino.
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
        // Columna derrumbada.
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

    append_ruin_details(&mut objects);
    append_outer_ruins(&mut objects);
    append_maya_complex(&mut objects);
    append_terrain(&mut objects, TerrainConfig::temple(40));
    let bvh = Bvh::build(&objects);

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
                position: Vec3::new(0.0, 8.0, 1.0),
                color: LinearColor::new(0.42, 0.60, 0.82),
                intensity: 62.0,
            },
            PointLight {
                position: Vec3::new(-5.5, 7.5, 1.5),
                color: LinearColor::new(1.0, 0.66, 0.38),
                intensity: 38.0,
            },
            PointLight {
                position: Vec3::new(0.0, 8.0, -10.5),
                color: LinearColor::new(1.0, 0.72, 0.42),
                intensity: 18.0,
            },
        ],
        skybox: Skybox { texture_id: 8 },
        bvh,
    })
}

fn append_ruin_details(objects: &mut Vec<Box<dyn RayIntersect>>) {
    // La parte faltante del arco yace entre la vegetación del lado derecho.
    objects.push(Box::new(Cube::new(
        Vec3::new(4.42, -0.68, -3.35),
        Vec3::new(5.78, -0.12, -2.30),
        0,
    )));
    objects.push(Box::new(Cube::new(
        Vec3::new(4.55, -0.11, -3.26),
        Vec3::new(5.52, 0.02, -2.42),
        1,
    )));

    // Manchas irregulares cubren el frente del dintel conservado.
    objects.push(Box::new(Cube::new(
        Vec3::new(-2.58, 3.12, -6.14),
        Vec3::new(-1.72, 3.48, -6.02),
        1,
    )));
    objects.push(Box::new(Cube::new(
        Vec3::new(1.46, 2.92, -6.14),
        Vec3::new(2.28, 3.18, -6.02),
        1,
    )));

    // Enredaderas que caen de las columnas y los muros.
    for (min, max) in [
        (Vec3::new(-2.48, 0.55, -6.08), Vec3::new(-2.30, 2.55, -5.96)),
        (Vec3::new(2.24, 1.45, -6.08), Vec3::new(2.43, 2.68, -5.96)),
        (Vec3::new(-4.06, 0.18, -6.62), Vec3::new(-3.92, 1.54, -6.46)),
        (Vec3::new(3.82, 0.32, -7.82), Vec3::new(3.96, 1.72, -7.66)),
    ] {
        objects.push(Box::new(Cube::new(min, max, 1)));
    }

    // Fragmentos desprendidos que rompen la simetría del acceso.
    for (min, max, material) in [
        (
            Vec3::new(-5.45, -0.68, -6.55),
            Vec3::new(-4.72, -0.12, -5.85),
            0,
        ),
        (
            Vec3::new(-5.18, -0.68, -5.72),
            Vec3::new(-4.48, -0.28, -5.12),
            1,
        ),
        (
            Vec3::new(4.65, -0.68, -4.25),
            Vec3::new(5.42, -0.06, -3.58),
            0,
        ),
        (
            Vec3::new(5.18, -0.68, -5.18),
            Vec3::new(5.82, -0.22, -4.58),
            1,
        ),
        (
            Vec3::new(-3.42, -0.68, -7.62),
            Vec3::new(-2.92, -0.18, -7.08),
            0,
        ),
        (
            Vec3::new(3.02, -0.68, -3.18),
            Vec3::new(3.62, -0.14, -2.62),
            1,
        ),
    ] {
        objects.push(Box::new(Cube::new(min, max, material)));
    }

    // Arbustos escalonados enmarcan la ruina sin cerrar la vista central.
    for (min, max) in [
        (
            Vec3::new(-6.45, -0.68, -7.35),
            Vec3::new(-5.35, 0.42, -6.25),
        ),
        (
            Vec3::new(-6.72, -0.68, -5.95),
            Vec3::new(-5.82, 0.12, -5.02),
        ),
        (Vec3::new(-5.98, 0.18, -7.08), Vec3::new(-5.10, 0.88, -6.20)),
        (Vec3::new(5.32, -0.68, -7.52), Vec3::new(6.42, 0.38, -6.42)),
        (Vec3::new(5.75, -0.68, -6.18), Vec3::new(6.62, 0.08, -5.28)),
        (Vec3::new(5.02, 0.14, -7.18), Vec3::new(5.92, 0.82, -6.30)),
    ] {
        objects.push(Box::new(Cube::new(min, max, 8)));
    }
}

fn append_outer_ruins(objects: &mut Vec<Box<dyn RayIntersect>>) {
    // Un acceso derrumbado abre el recorrido hacia el patio principal.
    for (min, max, material) in [
        (
            Vec3::new(-4.55, -0.68, -0.55),
            Vec3::new(-3.58, 2.15, 0.18),
            0,
        ),
        (
            Vec3::new(3.58, -0.68, -0.55),
            Vec3::new(4.55, 1.78, 0.18),
            0,
        ),
        (
            Vec3::new(-4.78, -0.68, -0.72),
            Vec3::new(-3.36, -0.35, 0.34),
            1,
        ),
        (
            Vec3::new(3.36, -0.68, -0.72),
            Vec3::new(4.78, -0.35, 0.34),
            1,
        ),
        (
            Vec3::new(-4.72, 1.88, -0.62),
            Vec3::new(-1.78, 2.42, 0.24),
            0,
        ),
        (Vec3::new(1.82, 1.72, -0.62), Vec3::new(4.66, 2.24, 0.24), 0),
        (
            Vec3::new(-4.42, 0.78, 0.19),
            Vec3::new(-4.18, 1.92, 0.31),
            1,
        ),
        (Vec3::new(3.72, 0.42, 0.19), Vec3::new(3.96, 1.54, 0.31), 1),
    ] {
        objects.push(Box::new(Cube::new(min, max, material)));
    }

    // El perímetro exterior se conserva solo en tramos separados.
    for (min, max, material) in [
        (
            Vec3::new(-9.2, -0.68, -13.1),
            Vec3::new(-5.7, 1.18, -12.45),
            1,
        ),
        (
            Vec3::new(-5.45, -0.68, -13.0),
            Vec3::new(-1.7, 0.48, -12.42),
            0,
        ),
        (
            Vec3::new(1.35, -0.68, -13.08),
            Vec3::new(4.85, 0.82, -12.43),
            0,
        ),
        (
            Vec3::new(5.12, -0.68, -13.15),
            Vec3::new(9.15, 1.42, -12.46),
            1,
        ),
        (
            Vec3::new(-9.35, -0.68, -12.35),
            Vec3::new(-8.62, 1.55, -8.95),
            0,
        ),
        (
            Vec3::new(-9.28, -0.68, -8.62),
            Vec3::new(-8.58, 0.72, -5.28),
            1,
        ),
        (
            Vec3::new(-9.18, -0.68, -4.95),
            Vec3::new(-8.48, 1.12, -1.82),
            0,
        ),
        (
            Vec3::new(8.58, -0.68, -12.42),
            Vec3::new(9.34, 1.88, -9.08),
            1,
        ),
        (
            Vec3::new(8.52, -0.68, -8.76),
            Vec3::new(9.25, 0.92, -5.42),
            0,
        ),
        (
            Vec3::new(8.45, -0.68, -5.08),
            Vec3::new(9.18, 1.32, -1.72),
            1,
        ),
    ] {
        objects.push(Box::new(Cube::new(min, max, material)));
    }

    // Columnas aisladas sugieren que el complejo continuaba fuera del santuario.
    for (x, z, height, material) in [
        (-7.15, -10.7, 2.65, 0),
        (7.18, -10.45, 2.15, 1),
        (-7.32, -6.72, 1.72, 1),
        (7.38, -6.48, 2.42, 0),
        (-7.05, -2.88, 2.18, 0),
        (7.12, -2.72, 1.58, 1),
    ] {
        objects.push(Box::new(Cube::new(
            Vec3::new(x - 0.48, -0.68, z - 0.48),
            Vec3::new(x + 0.48, -0.38, z + 0.48),
            material,
        )));
        objects.push(Box::new(Cube::new(
            Vec3::new(x - 0.31, -0.38, z - 0.31),
            Vec3::new(x + 0.31, height, z + 0.31),
            material,
        )));
    }

    // Árboles grandes cierran el horizonte con pocas cajas amplias.
    for (x, z, height, crown_size) in [
        (-10.4, -11.2, 3.8, 1.8),
        (10.2, -10.5, 4.2, 2.0),
        (-10.6, -5.0, 3.2, 1.7),
        (10.5, -4.2, 3.6, 1.8),
        (-9.8, 1.8, 2.9, 1.6),
        (9.9, 2.4, 3.1, 1.7),
    ] {
        objects.push(Box::new(Cube::new(
            Vec3::new(x - 0.24, -0.68, z - 0.24),
            Vec3::new(x + 0.24, height, z + 0.24),
            7,
        )));
        objects.push(Box::new(Cube::new(
            Vec3::new(x - crown_size * 0.5, height - 0.72, z - crown_size * 0.5),
            Vec3::new(x + crown_size * 0.5, height + 0.72, z + crown_size * 0.5),
            8,
        )));
    }
}

fn append_maya_complex(objects: &mut Vec<Box<dyn RayIntersect>>) {
    // La pirámide posterior domina el eje ceremonial del conjunto.
    append_stepped_platform(objects, 0.0, -16.0, 12.8, 9.2, 5, 0.76);

    // La escalinata central asciende desde la plaza hasta el santuario superior.
    for step in 0..9 {
        let height = -0.62 + (step + 1) as f32 * 0.43;
        let front_z = -9.35 - step as f32 * 0.46;
        let half_width = 1.62 - step as f32 * 0.055;
        objects.push(Box::new(Cube::new(
            Vec3::new(-half_width, -0.66, front_z - 0.58),
            Vec3::new(half_width, height, front_z),
            if step % 4 == 0 { 1 } else { 0 },
        )));
    }

    // El templo y su crestería producen la silueta vertical típica del complejo.
    for (min, max, material) in [
        (
            Vec3::new(-3.05, 3.08, -18.42),
            Vec3::new(3.05, 3.46, -13.72),
            0,
        ),
        (
            Vec3::new(-2.82, 3.46, -18.10),
            Vec3::new(-1.72, 5.52, -14.18),
            0,
        ),
        (
            Vec3::new(1.72, 3.46, -18.10),
            Vec3::new(2.82, 5.52, -14.18),
            0,
        ),
        (
            Vec3::new(-2.82, 5.02, -18.08),
            Vec3::new(2.82, 5.58, -14.20),
            0,
        ),
        (
            Vec3::new(-2.18, 5.58, -17.70),
            Vec3::new(2.18, 6.32, -14.72),
            0,
        ),
        (
            Vec3::new(-1.42, 6.32, -17.35),
            Vec3::new(1.42, 6.92, -15.08),
            0,
        ),
        (
            Vec3::new(-0.68, 3.46, -14.16),
            Vec3::new(0.68, 4.82, -13.96),
            3,
        ),
    ] {
        objects.push(Box::new(Cube::new(min, max, material)));
    }

    // Un remate de oro reflectivo corona el eje simétrico del santuario.
    objects.push(Box::new(Cube::new(
        Vec3::new(-0.68, 6.92, -16.82),
        Vec3::new(0.68, 7.20, -15.62),
        3,
    )));
    objects.push(Box::new(Cube::new(
        Vec3::new(-0.30, 7.20, -16.58),
        Vec3::new(0.30, 7.72, -15.86),
        3,
    )));

    // Pares de musgo conservan el desgaste sin romper la simetría del santuario.
    for (min, max) in [
        (
            Vec3::new(-2.55, 4.18, -14.17),
            Vec3::new(-2.05, 4.78, -14.05),
        ),
        (Vec3::new(2.05, 4.18, -14.17), Vec3::new(2.55, 4.78, -14.05)),
    ] {
        objects.push(Box::new(Cube::new(min, max, 1)));
    }

    // Dos basamentos menores amplían la plaza sin competir con la pirámide central.
    append_stepped_platform(objects, -10.7, -14.5, 6.2, 6.8, 3, 0.58);
    append_stepped_platform(objects, 10.5, -14.2, 5.8, 6.4, 3, 0.54);

    // Estelas y altares bajos marcan los bordes del espacio ceremonial.
    for (x, z, height, material) in [
        (-5.8, -10.2, 2.15, 0),
        (5.9, -10.0, 1.82, 1),
        (-6.4, -5.4, 1.45, 1),
        (6.5, -5.2, 1.68, 0),
        (-5.6, -1.6, 1.22, 0),
        (5.7, -1.4, 1.38, 1),
    ] {
        objects.push(Box::new(Cube::new(
            Vec3::new(x - 0.42, -0.66, z - 0.24),
            Vec3::new(x + 0.42, height, z + 0.24),
            material,
        )));
        objects.push(Box::new(Cube::new(
            Vec3::new(x - 0.62, -0.66, z - 0.46),
            Vec3::new(x + 0.62, -0.38, z + 0.46),
            material,
        )));
    }
}

fn append_stepped_platform(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    center_x: f32,
    center_z: f32,
    width: f32,
    depth: f32,
    levels: usize,
    rise: f32,
) {
    for level in 0..levels {
        let inset = level as f32 * 0.72;
        let half_width = width * 0.5 - inset;
        let half_depth = depth * 0.5 - inset;
        let bottom = -0.66 + level as f32 * rise;
        let top = bottom + rise + 0.04;
        objects.push(Box::new(Cube::new(
            Vec3::new(center_x - half_width, bottom, center_z - half_depth),
            Vec3::new(center_x + half_width, top, center_z + half_depth),
            if level + 1 == levels { 1 } else { 0 },
        )));
    }
}
