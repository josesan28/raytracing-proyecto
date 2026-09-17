use crate::color::Color;
use crate::material::{Intersect, RayIntersect};
use crate::math::Vec3;
use crate::sphere::Sphere;

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Sphere],
    background: Color,
) -> Color {
    let mut closest: Option<Intersect> = None;

    for object in objects {
        let intersect = object.ray_intersect(ray_origin, ray_direction);

        if intersect.is_intersecting {
            let is_closer = match &closest {
                Some(current) => intersect.distance < current.distance,
                None => true,
            };

            if is_closer {
                closest = Some(intersect);
            }
        }
    }

    closest
        .map(|intersect| intersect.material.diffuse)
        .unwrap_or(background)
}

pub fn render(width: usize, height: usize, objects: &[Sphere], background: Color) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(width * height);
    let aspect_ratio = width as f32 / height as f32;
    let ray_origin = Vec3::new(0.0, 0.0, 0.0);

    for y in 0..height {
        for x in 0..width {
            let screen_x = (2.0 * (x as f32 + 0.5)) / width as f32 - 1.0;
            let screen_y = 1.0 - (2.0 * (y as f32 + 0.5)) / height as f32;
            let screen_x = screen_x * aspect_ratio;
            let ray_direction = Vec3::new(screen_x, screen_y, -1.0).normalize();

            pixels.push(cast_ray(&ray_origin, &ray_direction, objects, background));
        }
    }

    pixels
}
