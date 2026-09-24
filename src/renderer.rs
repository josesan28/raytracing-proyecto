use crate::camera::Camera;
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

pub fn render(framebuffer: &mut Framebuffer, camera: &Camera, objects: &[Sphere]) {
    framebuffer.clear();

    let width = framebuffer.width;
    let height = framebuffer.height;
    let ray_origin = camera.origin();
    let background = framebuffer.background_color();

    for y in 0..height {
        for x in 0..width {
            let ray_direction = camera.ray_direction(x, y, width, height);

            let color = trace_ray(&ray_origin, &ray_direction, objects, background);
            framebuffer.set_current_color(color);
            framebuffer.set_pixel(x, y);
        }
    }
}
