use crate::camera::Camera;
use crate::color::{Color, LinearColor};
use crate::framebuffer::Framebuffer;
use crate::material::{Intersect, RAY_EPSILON};
use crate::math::Vec3;
use crate::scene::Scene;

const AMBIENT_INTENSITY: f32 = 0.19;
const MAX_DEPTH: u32 = 3;

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

    material.albedo.modulate(texture_color(intersection, scene))
}

fn texture_color(intersection: &Intersect, scene: &Scene) -> LinearColor {
    let material = &scene.materials[intersection.material_id];
    let scaled_uv = crate::math::Vec2::new(
        intersection.uv.u * material.texture_scale.u,
        intersection.uv.v * material.texture_scale.v,
    );

    material
        .texture_id
        .map(|texture_id| scene.textures[texture_id].sample_nearest(scaled_uv))
        .unwrap_or(LinearColor::new(1.0, 1.0, 1.0))
}

fn shading_normal(intersection: &Intersect, scene: &Scene) -> Vec3 {
    let material = &scene.materials[intersection.material_id];
    let Some(normal_map_id) = material.normal_map_id else {
        return intersection.normal;
    };
    let scaled_uv = crate::math::Vec2::new(
        intersection.uv.u * material.texture_scale.u,
        intersection.uv.v * material.texture_scale.v,
    );
    let sample = scene.textures[normal_map_id].sample_nearest(scaled_uv);
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

    blocker.is_intersecting && scene.materials[blocker.material_id].transparency < 0.5
}

fn shade(intersection: &Intersect, ray_direction: &Vec3, scene: &Scene) -> LinearColor {
    let material = &scene.materials[intersection.material_id];
    let albedo = surface_color(intersection, scene);
    let shading_normal = shading_normal(intersection, scene);
    let view_direction = (*ray_direction * -1.0).normalize();
    let emitted = material
        .emission
        .modulate(texture_color(intersection, scene));
    let mut result = albedo.scale(AMBIENT_INTENSITY).add(emitted);

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

fn trace_ray_linear(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    scene: &Scene,
    depth: u32,
) -> LinearColor {
    let intersection = closest_intersection(ray_origin, ray_direction, scene, f32::INFINITY);

    if !intersection.is_intersecting {
        return scene.textures[scene.skybox.texture_id].sample_equirectangular(*ray_direction);
    }

    let local_color = shade(&intersection, ray_direction, scene);
    let material = &scene.materials[intersection.material_id];
    let reflectivity = material.reflectivity.clamp(0.0, 1.0);
    let transparency = material.transparency.clamp(0.0, 1.0 - reflectivity);

    if (reflectivity <= 0.0 && transparency <= 0.0) || depth >= MAX_DEPTH {
        return local_color;
    }

    let surface_normal = shading_normal(&intersection, scene);
    let cosine = (*ray_direction * -1.0).dot(surface_normal).clamp(0.0, 1.0);
    let refractive_index = material.refractive_index.max(f32::EPSILON);
    let base_fresnel = ((1.0 - refractive_index) / (1.0 + refractive_index)).powi(2);
    let fresnel = base_fresnel + (1.0 - base_fresnel) * (1.0 - cosine).powi(5);
    let mut reflected_weight = reflectivity + transparency * fresnel;
    let mut refracted_weight = transparency * (1.0 - fresnel);
    let local_weight = 1.0 - reflectivity - transparency;

    let index_ratio = if intersection.front_face {
        1.0 / refractive_index
    } else {
        refractive_index
    };
    let refracted_direction = ray_direction.refract(surface_normal, index_ratio);

    if refracted_direction.is_none() {
        reflected_weight += refracted_weight;
        refracted_weight = 0.0;
    }

    let mut result = local_color.scale(local_weight);

    if reflected_weight > 0.0 {
        let reflected_direction = ray_direction.reflect(surface_normal).normalize();
        let offset_normal = if reflected_direction.dot(intersection.normal) >= 0.0 {
            intersection.normal
        } else {
            intersection.normal * -1.0
        };
        let reflected_origin = intersection.point + offset_normal * RAY_EPSILON;
        let reflected_color =
            trace_ray_linear(&reflected_origin, &reflected_direction, scene, depth + 1);
        result = result.add(reflected_color.scale(reflected_weight));
    }

    if let Some(refracted_direction) = refracted_direction {
        let refracted_direction = refracted_direction.normalize();
        let refracted_origin = intersection.point - intersection.normal * RAY_EPSILON;
        let refracted_color =
            trace_ray_linear(&refracted_origin, &refracted_direction, scene, depth + 1);
        result = result.add(refracted_color.scale(refracted_weight));
    }

    result
}

pub fn trace_ray(ray_origin: &Vec3, ray_direction: &Vec3, scene: &Scene) -> Color {
    trace_ray_linear(ray_origin, ray_direction, scene, 0).to_color()
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
