use crate::color::Color;
use crate::material::Material;
use crate::math::Vec3;
use crate::sphere::Sphere;

pub fn zbuffer_scene() -> Vec<Sphere> {
    let ivory_material = Material::new(Color::new(245, 245, 235));
    let red_material = Material::new(Color::new(220, 45, 55));
    let green_material = Material::new(Color::new(55, 180, 100));

    vec![
        // La esfera cercana aparece primero en el arreglo a propósito.
        Sphere::new(Vec3::new(0.0, 0.0, -3.0), 1.0, ivory_material),
        // Esta esfera está más lejos y se superpone con la anterior.
        Sphere::new(Vec3::new(0.0, 0.0, -4.0), 1.0, red_material),
        Sphere::new(Vec3::new(1.35, -0.35, -4.2), 0.55, green_material),
    ]
}
