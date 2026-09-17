use crate::color::Color;
use crate::material::Material;
use crate::math::Vec3;
use crate::sphere::Sphere;

pub fn demo_scene() -> Vec<Sphere> {
    let red_material = Material::new(Color::new(220, 45, 55));
    let blue_material = Material::new(Color::new(45, 105, 220));
    let green_material = Material::new(Color::new(55, 180, 100));

    vec![
        // Esta esfera azul queda detrás de la esfera roja.
        Sphere::new(Vec3::new(-0.20, 0.0, -3.8), 1.0, blue_material),
        // Las dos se superponen; el material rojo debe verse en la zona común.
        Sphere::new(Vec3::new(0.25, 0.0, -3.0), 0.9, red_material),
        Sphere::new(Vec3::new(1.35, -0.35, -4.2), 0.55, green_material),
    ]
}
