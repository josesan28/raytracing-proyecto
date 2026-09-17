use crate::color::Color;
use crate::math::Vec3;
use crate::sphere::Sphere;

const BACKGROUND: Color = [5, 5, 8];
const WHITE: Color = [248, 248, 250];
const SOFT_WHITE: Color = [224, 228, 235];
const LIGHT_GRAY: Color = [180, 185, 194];
const BLACK: Color = [18, 18, 21];
const RED: Color = [224, 35, 48];
const DARK_RED: Color = [160, 18, 30];

pub struct Scene {
    pub objects: Vec<Sphere>,
    pub background: Color,
}

fn sphere(x: f32, y: f32, z: f32, radius: f32, color: Color) -> Sphere {
    Sphere::new(Vec3::new(x, y, z), radius, color)
}

fn add_sphere_chain(
    objects: &mut Vec<Sphere>,
    start: (f32, f32),
    end: (f32, f32),
    z: f32,
    radius: f32,
    color: Color,
    count: usize,
) {
    for index in 0..count {
        let progress = index as f32 / (count - 1) as f32;
        let x = start.0 + (end.0 - start.0) * progress;
        let y = start.1 + (end.1 - start.1) * progress;
        objects.push(sphere(x, y, z, radius, color));
    }
}

pub fn baymax_scene() -> Scene {
    let mut objects = vec![
        // Torso y abdomen redondeados.
        sphere(0.0, 0.28, -7.30, 1.30, WHITE),
        sphere(0.0, -0.72, -7.30, 1.63, WHITE),
        sphere(0.0, -1.28, -7.30, 1.42, WHITE),
        // Cuello.
        sphere(0.0, 0.98, -7.05, 0.67, SOFT_WHITE),
        // Piernas y pies detrás del cuerpo.
        sphere(-0.58, -2.05, -7.00, 0.62, SOFT_WHITE),
        sphere(0.58, -2.05, -7.00, 0.62, SOFT_WHITE),
        sphere(-0.58, -2.48, -6.70, 0.56, WHITE),
        sphere(0.58, -2.48, -6.70, 0.56, WHITE),
        // Cabeza redondeada formada por tres esferas superpuestas.
        sphere(-0.20, 1.62, -6.40, 0.57, WHITE),
        sphere(0.20, 1.62, -6.40, 0.57, WHITE),
        sphere(0.0, 1.66, -6.42, 0.58, WHITE),
        // Ojos.
        sphere(-0.25, 1.65, -5.80, 0.078, BLACK),
        sphere(0.25, 1.65, -5.80, 0.078, BLACK),
        // Puerto circular del pecho.
        sphere(0.42, 0.35, -5.82, 0.145, LIGHT_GRAY),
        sphere(0.42, 0.35, -5.65, 0.095, WHITE),
    ];

    // Línea que conecta los ojos.
    add_sphere_chain(
        &mut objects,
        (-0.20, 1.65),
        (0.20, 1.65),
        -5.84,
        0.026,
        BLACK,
        10,
    );

    // Brazo derecho, colocado detrás del torso.
    add_sphere_chain(
        &mut objects,
        (1.02, 0.30),
        (1.43, -1.30),
        -7.00,
        0.43,
        SOFT_WHITE,
        7,
    );
    objects.extend([
        sphere(1.44, -1.54, -6.35, 0.34, WHITE),
        sphere(1.56, -1.75, -6.05, 0.17, WHITE),
        sphere(1.43, -1.84, -6.05, 0.17, WHITE),
        sphere(1.30, -1.74, -6.05, 0.17, WHITE),
    ]);

    // Brazo izquierdo cruzado frente al abdomen.
    add_sphere_chain(
        &mut objects,
        (-1.12, -0.10),
        (-0.12, -0.55),
        -5.48,
        0.36,
        SOFT_WHITE,
        9,
    );
    objects.push(sphere(-0.04, -0.53, -4.88, 0.25, SOFT_WHITE));

    // Palito de la paleta, frente a la mano.
    add_sphere_chain(
        &mut objects,
        (0.02, -0.62),
        (0.02, 0.32),
        -3.90,
        0.050,
        LIGHT_GRAY,
        18,
    );

    // Dulce rojo y detalles de volumen.
    objects.extend([
        sphere(0.02, 0.43, -3.82, 0.15, RED),
        sphere(0.07, 0.39, -3.66, 0.058, DARK_RED),
        sphere(-0.03, 0.47, -3.64, 0.034, WHITE),
    ]);

    Scene {
        objects,
        background: BACKGROUND,
    }
}
