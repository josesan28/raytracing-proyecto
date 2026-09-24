use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::material::Intersect;
use crate::math::Vec3;
use crate::scene::Scene;

pub fn trace_ray(ray_origin: &Vec3, ray_direction: &Vec3, scene: &Scene) -> Color {
    let mut intersect = Intersect::empty();
    let mut nearest_distance = f32::INFINITY;

    for object in &scene.objects {
        let candidate = object.ray_intersect(ray_origin, ray_direction);

        if candidate.is_intersecting && candidate.distance < nearest_distance {
            nearest_distance = candidate.distance;
            intersect = candidate;
        }
    }

    if !intersect.is_intersecting {
        return scene.skybox.color;
    }

    scene.materials[intersect.material_id].albedo.to_color()
}

pub fn render(framebuffer: &mut Framebuffer, camera: &Camera, scene: &Scene) {
    framebuffer.clear();

    let width = framebuffer.width;
    let height = framebuffer.height;
    let ray_origin = camera.origin();

    for y in 0..height {
        for x in 0..width {
            let ray_direction = camera.ray_direction(x, y, width, height);
            let color = trace_ray(&ray_origin, &ray_direction, scene);
            framebuffer.set_current_color(color);
            framebuffer.set_pixel(x, y);
        }
    }
}
