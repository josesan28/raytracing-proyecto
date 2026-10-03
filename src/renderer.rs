use crate::camera::Camera;
use crate::color::LinearColor;
use crate::framebuffer::Framebuffer;
use crate::material::{Intersect, RAY_EPSILON};
use crate::math::Vec3;
use crate::scene::Scene;

const AMBIENT_INTENSITY: f32 = 0.19;
const MAX_DEPTH: u32 = 3;
const MIN_RAY_CONTRIBUTION: f32 = 1.0 / 4096.0;
const SUN_DIRECTION: Vec3 = Vec3::new(-0.363, -0.038, -0.931);

#[derive(Clone, Copy)]
pub struct RenderOptions {
    pub(crate) max_depth: u32,
    pub shadows: bool,
    pub reflections: bool,
    pub refractions: bool,
    pub normal_maps: bool,
    pub antialias: bool,
}

pub const FINAL_OPTIONS: RenderOptions = RenderOptions {
    max_depth: MAX_DEPTH,
    shadows: true,
    reflections: true,
    refractions: true,
    normal_maps: true,
    antialias: true,
};

pub const ECONOMY_OPTIONS: RenderOptions = RenderOptions {
    shadows: false,
    reflections: false,
    refractions: false,
    normal_maps: false,
    antialias: false,
    ..FINAL_OPTIONS
};

fn closest_intersection(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    scene: &Scene,
    max_distance: f32,
) -> Intersect {
    scene
        .bvh
        .closest_intersection(&scene.objects, ray_origin, ray_direction, max_distance)
}

fn sky_color(ray_direction: Vec3, scene: &Scene) -> LinearColor {
    let sky = scene.textures[scene.skybox.texture_id].sample_equirectangular(ray_direction);
    let alignment = ray_direction.dot(SUN_DIRECTION).clamp(-1.0, 1.0);
    let halo_base = ((alignment - 0.985) / 0.015).clamp(0.0, 1.0);
    let halo = halo_base * halo_base * halo_base;
    let disk_base = ((alignment - 0.99935) / 0.00065).clamp(0.0, 1.0);
    let disk = disk_base.sqrt().sqrt();
    let sun = LinearColor::new(0.78, 0.94, 0.58);

    sky.add(sun.scale(halo * 0.48 + disk * 1.65))
}

fn texture_color(intersection: &Intersect, scene: &Scene) -> LinearColor {
    let material = &scene.materials[intersection.material_id];
    let scaled_uv = crate::math::Vec2::new(
        intersection.uv.u * material.texture_scale.u,
        intersection.uv.v * material.texture_scale.v,
    );

    material
        .texture_id
        .map(|texture_id| scene.textures[texture_id].sample_bilinear(scaled_uv))
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
    let sample = scene.textures[normal_map_id].sample_bilinear(scaled_uv);
    let tangent_normal = Vec3::new(sample.r, sample.g, sample.b);

    (intersection.tangent * tangent_normal.x
        + intersection.bitangent * tangent_normal.y
        + intersection.normal * tangent_normal.z)
        .normalize()
}

fn is_in_shadow(
    intersection: &Intersect,
    light_direction: Vec3,
    light_distance: f32,
    scene: &Scene,
) -> bool {
    let shadow_origin = intersection.point + intersection.normal * RAY_EPSILON;
    scene.bvh.is_occluded(
        &scene.objects,
        &scene.materials,
        &shadow_origin,
        &light_direction,
        light_distance - RAY_EPSILON,
    )
}

fn shade(
    intersection: &Intersect,
    ray_direction: &Vec3,
    scene: &Scene,
    options: RenderOptions,
    shading_normal: Vec3,
) -> LinearColor {
    let material = &scene.materials[intersection.material_id];
    let texture = texture_color(intersection, scene);
    let albedo = material.albedo.modulate(texture);
    let view_direction = *ray_direction * -1.0;
    let emitted = material.emission.modulate(texture);
    let mut result = albedo.scale(AMBIENT_INTENSITY).add(emitted);

    for light in &scene.lights {
        let to_light = light.position - intersection.point;
        let distance_squared = to_light.dot(to_light);
        let light_distance = distance_squared.sqrt();
        let light_direction = to_light / light_distance;
        let normal_light = shading_normal.dot(light_direction).max(0.0);

        if normal_light <= 0.0
            || (options.shadows
                && is_in_shadow(intersection, light_direction, light_distance, scene))
        {
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
    options: RenderOptions,
    contribution: f32,
) -> LinearColor {
    let intersection = closest_intersection(ray_origin, ray_direction, scene, f32::INFINITY);

    if !intersection.is_intersecting {
        return sky_color(*ray_direction, scene);
    }

    let surface_normal = if options.normal_maps {
        shading_normal(&intersection, scene)
    } else {
        intersection.normal
    };
    let local_color = shade(&intersection, ray_direction, scene, options, surface_normal);
    let material = &scene.materials[intersection.material_id];
    let reflectivity = if options.reflections {
        material.reflectivity.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let transparency = if options.refractions {
        material.transparency.clamp(0.0, 1.0 - reflectivity)
    } else {
        0.0
    };

    if (reflectivity <= 0.0 && transparency <= 0.0) || depth >= options.max_depth {
        return local_color;
    }

    let cosine = (*ray_direction * -1.0).dot(surface_normal).clamp(0.0, 1.0);
    let refractive_index = material.refractive_index.max(f32::EPSILON);
    let base_fresnel = ((1.0 - refractive_index) / (1.0 + refractive_index)).powi(2);
    let fresnel = if options.reflections {
        base_fresnel + (1.0 - base_fresnel) * (1.0 - cosine).powi(5)
    } else {
        0.0
    };
    let mut reflected_weight = reflectivity + transparency * fresnel;
    let mut refracted_weight = transparency * (1.0 - fresnel);
    let mut local_weight = 1.0 - reflectivity - transparency;

    let index_ratio = if intersection.front_face {
        1.0 / refractive_index
    } else {
        refractive_index
    };
    let refracted_direction = ray_direction.refract(surface_normal, index_ratio);

    if refracted_direction.is_none() {
        if options.reflections {
            reflected_weight += refracted_weight;
        } else {
            // Sin reflexión, usa sombreado local ante reflexión interna total.
            local_weight += refracted_weight;
        }
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
        let reflected_contribution = contribution * reflected_weight;
        if reflected_contribution >= MIN_RAY_CONTRIBUTION {
            let reflected_color = trace_ray_linear(
                &reflected_origin,
                &reflected_direction,
                scene,
                depth + 1,
                options,
                reflected_contribution,
            );
            result = result.add(reflected_color.scale(reflected_weight));
        }
    }

    if let Some(refracted_direction) = refracted_direction {
        let refracted_direction = refracted_direction.normalize();
        let refracted_origin = intersection.point - intersection.normal * RAY_EPSILON;
        let refracted_contribution = contribution * refracted_weight;
        if refracted_contribution >= MIN_RAY_CONTRIBUTION {
            let refracted_color = trace_ray_linear(
                &refracted_origin,
                &refracted_direction,
                scene,
                depth + 1,
                options,
                refracted_contribution,
            );
            result = result.add(refracted_color.scale(refracted_weight));
        }
    }

    result
}

pub fn render(framebuffer: &mut Framebuffer, camera: &Camera, scene: &Scene) {
    render_with_options(
        framebuffer,
        camera,
        scene,
        RenderOptions {
            antialias: false,
            ..FINAL_OPTIONS
        },
        &|| false,
    );
}

pub fn render_refined(framebuffer: &mut Framebuffer, camera: &Camera, scene: &Scene) {
    render_with_options(framebuffer, camera, scene, FINAL_OPTIONS, &|| false);
}

// La cancelación se comprueba entre tareas; nunca se publica un cuadro parcial.
pub fn render_with_options(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    scene: &Scene,
    options: RenderOptions,
    cancelled: &(dyn Fn() -> bool + Sync),
) -> bool {
    framebuffer.clear();
    render_rows(framebuffer, camera, scene, options, cancelled)
}

fn render_rows(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    scene: &Scene,
    options: RenderOptions,
    cancelled: &(dyn Fn() -> bool + Sync),
) -> bool {
    let width = framebuffer.width;
    let height = framebuffer.height;
    if height == 0 || width == 0 {
        return true;
    }
    let ray_origin = camera.origin();
    let perspective_scale = camera.perspective_scale();
    let aspect_ratio = width as f32 / height as f32;
    let screen_x: Vec<f32> = (0..width)
        .map(|x| ((2.0 * (x as f32 + 0.5)) / width as f32 - 1.0) * aspect_ratio * perspective_scale)
        .collect();
    let screen_y: Vec<f32> = (0..height)
        .map(|y| (1.0 - (2.0 * (y as f32 + 0.5)) / height as f32) * perspective_scale)
        .collect();
    // Cuatro subpíxeles dentro del píxel; no se desenfoca la imagen terminada.
    let offset_x = camera.right * (0.5 / width as f32 * aspect_ratio * perspective_scale);
    let offset_y = camera.up * (0.5 / height as f32 * perspective_scale);
    let offsets = [
        offset_x + offset_y,
        offset_x - offset_y,
        offset_y - offset_x,
        (offset_x + offset_y) * -1.0,
    ];
    let rows_per_task = if options.antialias { 1 } else { 4 };
    let worker_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .saturating_sub(1)
        .max(1)
        .min(height.div_ceil(rows_per_task));
    // Franjas pequeñas equilibran cielo barato y geometría con rayos secundarios.
    let pixels_per_task = rows_per_task * width;
    let tasks = std::sync::Mutex::new(
        framebuffer
            .pixels_mut()
            .chunks_mut(pixels_per_task)
            .enumerate(),
    );

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let screen_x = &screen_x;
            let screen_y = &screen_y;
            let tasks = &tasks;
            scope.spawn(move || {
                loop {
                    if cancelled() {
                        break;
                    }
                    // El bloqueo solo reparte franjas; el trazado usa píxeles exclusivos.
                    let task = tasks.lock().expect("Cola de render disponible").next();
                    let Some((task, pixels)) = task else { break };
                    let start_y = task * rows_per_task;
                    for (local_y, row) in pixels.chunks_mut(width).enumerate() {
                        let y = start_y + local_y;
                        for (x, pixel) in row.iter_mut().enumerate() {
                            let direction = camera.right * screen_x[x]
                                + camera.up * screen_y[y]
                                + camera.forward;
                            let color = if options.antialias {
                                let mut sum = LinearColor::default();
                                for offset in offsets {
                                    sum = sum.add(trace_ray_linear(
                                        &ray_origin,
                                        &(direction + offset).normalize(),
                                        scene,
                                        0,
                                        options,
                                        1.0,
                                    ));
                                }
                                sum.scale(0.25)
                            } else {
                                trace_ray_linear(
                                    &ray_origin,
                                    &direction.normalize(),
                                    scene,
                                    0,
                                    options,
                                    1.0,
                                )
                            };
                            *pixel = color.to_color();
                        }
                    }
                }
            });
        }
    });
    !cancelled()
}
