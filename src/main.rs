mod color;
mod framebuffer;
mod material;
mod math;
mod output;
mod renderer;
mod scene;
mod sphere;

use color::Color;
use framebuffer::Framebuffer;
use output::write_ppm;
use renderer::render;
use scene::zbuffer_scene;

fn main() -> std::io::Result<()> {
    let width = 800;
    let height = 600;
    let mut framebuffer = Framebuffer::new(width, height, Color::new(8, 10, 18));
    let objects = zbuffer_scene();
    render(&mut framebuffer, &objects);

    write_ppm("zbuffer.ppm", &framebuffer)?;
    println!("Render terminado: zbuffer.ppm");

    Ok(())
}
