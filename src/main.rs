mod camera;
mod color;
mod framebuffer;
mod material;
mod math;
mod output;
mod renderer;
mod scene;
mod sphere;

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use math::Vec3;
use output::write_ppm;
use renderer::render;
use scene::zbuffer_scene;

fn main() -> std::io::Result<()> {
    use std::f32::consts::PI;

    let width = 800;
    let height = 600;
    let mut framebuffer = Framebuffer::new(width, height, Color::new(0, 0, 0));
    framebuffer.set_background_color(Color::new(8, 10, 18));

    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -3.5),
        Vec3::new(0.0, 1.0, 0.0),
        PI / 3.0,
    );
    let objects = zbuffer_scene();
    render(&mut framebuffer, &camera, &objects);

    write_ppm("zbuffer.ppm", &framebuffer)?;
    println!("Render terminado: zbuffer.ppm");

    Ok(())
}
