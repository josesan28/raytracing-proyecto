use crate::cube::Cube;
use crate::material::{MaterialId, RayIntersect};
use crate::math::Vec3;

pub struct TerrainConfig {
    pub seed: u32,
    pub width: usize,
    pub depth: usize,
    pub origin_x: f32,
    pub origin_z: f32,
    pub cell_size: f32,
    pub base_height: f32,
    pub stone_material: MaterialId,
    pub ground_material: MaterialId,
}

impl TerrainConfig {
    pub fn temple(seed: u32) -> Self {
        Self {
            seed,
            width: 32,
            depth: 32,
            origin_x: -16.0,
            origin_z: -21.0,
            cell_size: 1.0,
            base_height: -1.5,
            stone_material: 0,
            ground_material: 6,
        }
    }
}

pub fn append_terrain(objects: &mut Vec<Box<dyn RayIntersect>>, config: TerrainConfig) {
    objects.reserve(config.width * config.depth);

    for row in 0..config.depth {
        for column in 0..config.width {
            let min_x = config.origin_x + column as f32 * config.cell_size;
            let min_z = config.origin_z + row as f32 * config.cell_size;
            let center_x = min_x + config.cell_size * 0.5;
            let center_z = min_z + config.cell_size * 0.5;

            let terrain = fractal_noise(config.seed, center_x * 0.16, center_z * 0.16, 4);
            let moisture = fractal_noise(
                config.seed ^ 0x9E37_79B9,
                center_x * 0.12,
                center_z * 0.12,
                3,
            );
            let relief = relief_outside_sanctuary(center_x, center_z);
            let top = -0.7 + terrain * 0.52 * relief;
            let moss_limit = 0.08 - relief * 0.26;
            let material_id = if moisture - terrain * 0.25 > moss_limit {
                config.ground_material
            } else {
                config.stone_material
            };

            objects.push(Box::new(Cube::new(
                Vec3::new(min_x, config.base_height, min_z),
                Vec3::new(min_x + config.cell_size, top, min_z + config.cell_size),
                material_id,
            )));
        }
    }
}

fn relief_outside_sanctuary(x: f32, z: f32) -> f32 {
    let distance_x = (x.abs() - 7.0).max(0.0);
    let distance_z = if z < -20.5 {
        -20.5 - z
    } else if z > 1.2 {
        z - 1.2
    } else {
        0.0
    };

    (distance_x.max(distance_z) / 3.0).min(1.0)
}

fn fractal_noise(seed: u32, x: f32, z: f32, octaves: usize) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..octaves {
        value += value_noise(
            seed.wrapping_add((octave as u32).wrapping_mul(0x9E37_79B9)),
            x * frequency,
            z * frequency,
        ) * amplitude;
        amplitude_sum += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / amplitude_sum
}

fn value_noise(seed: u32, x: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;
    let blend_x = smooth_step(x - x.floor());
    let blend_z = smooth_step(z - z.floor());

    let top = lerp(
        random_signed(seed, x0, z0),
        random_signed(seed, x1, z0),
        blend_x,
    );
    let bottom = lerp(
        random_signed(seed, x0, z1),
        random_signed(seed, x1, z1),
        blend_x,
    );

    lerp(top, bottom, blend_z)
}

fn smooth_step(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount
}

fn random_signed(seed: u32, x: i32, z: i32) -> f32 {
    let mut value =
        seed ^ (x as u32).wrapping_mul(0x85EB_CA6B) ^ (z as u32).wrapping_mul(0xC2B2_AE35);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32 * 2.0 - 1.0
}
