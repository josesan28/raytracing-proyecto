use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::material::{Intersect, RayIntersect};
use crate::math::Vec3;
use crate::sphere::Sphere;

pub fn trace_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Sphere],
    background: Color,
) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let candidate = object.ray_intersect(ray_origin, ray_direction);

        if candidate.is_intersecting && candidate.distance < zbuffer {
            zbuffer = candidate.distance;
            intersect = candidate;
        }
    }

    if !intersect.is_intersecting {
        return background;
    }

    intersect.material.diffuse
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere]) {
    framebuffer.clear();

    let width = framebuffer.width;
    let height = framebuffer.height;
    let aspect_ratio = width as f32 / height as f32;
    let ray_origin = Vec3::new(0.0, 0.0, 0.0);
    let background = framebuffer.background_color();

    for y in 0..height {
        for x in 0..width {
            let screen_x = (2.0 * (x as f32 + 0.5)) / width as f32 - 1.0;
            let screen_y = 1.0 - (2.0 * (y as f32 + 0.5)) / height as f32;
            let screen_x = screen_x * aspect_ratio;
            let ray_direction = Vec3::new(screen_x, screen_y, -1.0).normalize();

            let color = trace_ray(&ray_origin, &ray_direction, objects, background);
            framebuffer.set_current_color(color);
            framebuffer.point(x, y);
        }
    }
}
