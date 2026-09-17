mod color;
mod math;
mod output;
mod renderer;
mod scene;
mod sphere;

use output::{write_bmp, write_png};
use renderer::render;
use scene::baymax_scene;

fn main() -> std::io::Result<()> {
    let width = 600;
    let height = 750;
    let scene = baymax_scene();
    let pixels = render(width, height, &scene.objects, scene.background);

    write_bmp("baymax.bmp", width, height, &pixels)?;
    write_png("baymax.png", width, height, &pixels)?;
    println!(
        "Render terminado: baymax.bmp y baymax.png ({} esferas)",
        scene.objects.len()
    );

    Ok(())
}
