mod color;
mod material;
mod math;
mod output;
mod renderer;
mod scene;
mod sphere;

use color::Color;
use output::write_ppm;
use renderer::render;
use scene::zbuffer_scene;

fn main() -> std::io::Result<()> {
    let width = 800;
    let height = 600;
    let background = Color::new(8, 10, 18);
    let objects = zbuffer_scene();
    let pixels = render(width, height, &objects, background);

    write_ppm("zbuffer.ppm", width, height, &pixels)?;
    println!("Render terminado: zbuffer.ppm");

    Ok(())
}
