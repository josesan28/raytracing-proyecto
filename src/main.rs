mod camera;
mod color;
mod cube;
mod framebuffer;
mod material;
mod math;
mod output;
mod renderer;
mod scene;
#[allow(dead_code)]
mod sphere;
mod texture;

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use math::Vec3;
use output::{write_bmp, write_ppm};
use renderer::render;
use scene::temple_scene;

fn main() -> std::io::Result<()> {
    use std::f32::consts::PI;

    let width = 800;
    let height = 450;
    let mut framebuffer = Framebuffer::new(width, height, Color::new(0, 0, 0));
    framebuffer.set_background_color(Color::new(8, 10, 18));

    let camera = Camera::new(
        Vec3::new(3.65, 2.55, 2.15),
        Vec3::new(0.0, 0.75, -5.7),
        Vec3::new(0.0, 1.0, 0.0),
        PI / 3.5,
    );
    let scene = temple_scene()?;
    render(&mut framebuffer, &camera, &scene);

    write_ppm("temple_cubes.ppm", &framebuffer)?;
    write_bmp("temple_cubes.bmp", &framebuffer)?;
    println!("Render terminado: temple_cubes.ppm y temple_cubes.bmp");

    Ok(())
}
