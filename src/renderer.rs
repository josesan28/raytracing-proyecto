use crate::camera::Camera;
use crate::color::{Color, LinearColor};
use crate::framebuffer::Framebuffer;
use crate::material::{Intersect, RAY_EPSILON};
use crate::math::Vec3;
use crate::scene::Scene;

const AMBIENT_INTENSITY: f32 = 0.14;

fn closest_intersection(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    scene: &Scene,
    max_distance: f32,
) -> Intersect {
    let mut intersection = Intersect::empty();
    let mut nearest_distance = max_distance;

    for object in &scene.objects {
        let candidate = object.ray_intersect(ray_origin, ray_direction);

        if candidate.is_intersecting && candidate.distance < nearest_distance {
            nearest_distance = candidate.distance;
            intersection = candidate;
        }
    }

    intersection
}

fn surface_color(intersection: &Intersect, scene: &Scene) -> LinearColor {
    let material = &scene.materials[intersection.material_id];

    match material.texture_id {
        Some(texture_id) => material
            .albedo
            .modulate(scene.textures[texture_id].sample_nearest(intersection.uv)),
        None => material.albedo,
    }
}

fn shading_normal(intersection: &Intersect, scene: &Scene) -> Vec3 {
    let material = &scene.materials[intersection.material_id];
    let Some(normal_map_id) = material.normal_map_id else {
        return intersection.normal;
    };
    let sample = scene.textures[normal_map_id].sample_nearest(intersection.uv);
    let tangent_normal = Vec3::new(
        sample.r * 2.0 - 1.0,
        sample.g * 2.0 - 1.0,
        sample.b * 2.0 - 1.0,
    )
    .normalize();

    (intersection.tangent * tangent_normal.x
        + intersection.bitangent * tangent_normal.y
        + intersection.normal * tangent_normal.z)
        .normalize()
}

fn is_in_shadow(intersection: &Intersect, light_position: Vec3, scene: &Scene) -> bool {
    let to_light = light_position - intersection.point;
    let light_distance = to_light.length();
    let light_direction = to_light / light_distance;
    let shadow_origin = intersection.point + intersection.normal * RAY_EPSILON;
    let blocker = closest_intersection(
        &shadow_origin,
        &light_direction,
        scene,
        light_distance - RAY_EPSILON,
    );

    blocker.is_intersecting
}

fn shade(intersection: &Intersect, ray_direction: &Vec3, scene: &Scene) -> LinearColor {
    let material = &scene.materials[intersection.material_id];
    let albedo = surface_color(intersection, scene);
    let shading_normal = shading_normal(intersection, scene);
    let view_direction = (*ray_direction * -1.0).normalize();
    let mut result = albedo.scale(AMBIENT_INTENSITY);

    for light in &scene.lights {
        let to_light = light.position - intersection.point;
        let distance_squared = to_light.dot(to_light);
        let light_direction = to_light.normalize();
        let normal_light = shading_normal.dot(light_direction).max(0.0);

        if normal_light <= 0.0 || is_in_shadow(intersection, light.position, scene) {
            continue;
        }

        let attenuation = light.intensity / distance_squared.max(1.0);
        let radiance = light.color.scale(attenuation);
        let diffuse = albedo.modulate(radiance).scale(normal_light);

        let half_direction = (light_direction + view_direction).normalize();
        let specular_factor = shading_normal
            .dot(half_direction)
            .max(0.0)
            .powf(material.shininess)
            * material.specular;
        let specular = radiance.scale(specular_factor);

        result = result.add(diffuse).add(specular);
    }

    result
}

pub fn trace_ray(ray_origin: &Vec3, ray_direction: &Vec3, scene: &Scene) -> Color {
    let intersection = closest_intersection(ray_origin, ray_direction, scene, f32::INFINITY);

    if !intersection.is_intersecting {
        return scene.skybox.color;
    }

    shade(&intersection, ray_direction, scene).to_color()
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
