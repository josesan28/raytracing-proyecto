mod bounding_box;
mod bvh;
mod camera;
mod color;
mod cube;
mod framebuffer;
mod material;
mod math;
mod output;
mod procedural;
mod renderer;
mod scene;
#[allow(dead_code)]
mod sphere;
mod texture;
mod window;
#[cfg(target_os = "windows")]
mod window_renderer;

use std::fs;
use std::io::{self, BufRead, Write};

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use math::Vec3;
use output::{write_bmp, write_png, write_ppm};
use renderer::{render, render_refined};
use scene::{Scene, temple_scene};

const FINAL_RENDER_WIDTH: usize = 1600;
const FINAL_RENDER_HEIGHT: usize = 900;

fn main() -> std::io::Result<()> {
    let scene = temple_scene()?;
    let arguments: Vec<String> = std::env::args().collect();

    if arguments.iter().any(|argument| argument == "--render") {
        render_main_view(&scene)
    } else if arguments.iter().any(|argument| argument == "--interactive") {
        run_interactive(&scene)
    } else if arguments.iter().any(|argument| argument == "--animation") {
        render_animation(&scene)
    } else {
        window::run(&scene)
    }
}

fn interactive_camera() -> Camera {
    use std::f32::consts::PI;

    Camera::new(
        Vec3::new(11.5, 7.2, 12.5),
        Vec3::new(0.0, 1.45, -9.2),
        Vec3::new(0.0, 1.0, 0.0),
        PI / 3.5,
    )
}

fn run_interactive(scene: &Scene) -> std::io::Result<()> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut camera = interactive_camera();
    let mut framebuffer = Framebuffer::new(120, 68, Color::new(8, 10, 18));

    loop {
        render(&mut framebuffer, &camera, scene);
        display_in_terminal(&framebuffer, &camera)?;

        let mut command = String::new();
        if input.read_line(&mut command)? == 0 {
            break;
        }

        let mut should_exit = false;
        for key in command.trim().chars() {
            match key.to_ascii_lowercase() {
                'a' => camera.orbit(-0.12, 0.0),
                'd' => camera.orbit(0.12, 0.0),
                'w' => camera.orbit(0.0, 0.10),
                's' => camera.orbit(0.0, -0.10),
                '+' | '=' => camera.zoom(0.7),
                '-' | '_' => camera.zoom(-0.7),
                'r' => camera = interactive_camera(),
                'g' => save_interactive_view(scene, &camera)?,
                'q' => should_exit = true,
                _ => {}
            }
        }

        if should_exit {
            break;
        }
    }

    print!("\x1b[0m\x1b[2J\x1b[H");
    println!("Modo interactivo finalizado.");
    Ok(())
}

fn display_in_terminal(framebuffer: &Framebuffer, camera: &Camera) -> std::io::Result<()> {
    use std::fmt::Write as _;

    let mut preview = String::with_capacity(framebuffer.width * framebuffer.height * 12);
    preview.push_str("\x1b[2J\x1b[H");

    for y in (0..framebuffer.height).step_by(2) {
        for x in 0..framebuffer.width {
            let top = framebuffer.pixels()[y * framebuffer.width + x];
            let bottom_y = (y + 1).min(framebuffer.height - 1);
            let bottom = framebuffer.pixels()[bottom_y * framebuffer.width + x];
            write!(
                preview,
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                top.r, top.g, top.b, bottom.r, bottom.g, bottom.b
            )
            .expect("Escribir en una cadena no puede fallar");
        }
        preview.push_str("\x1b[0m\n");
    }

    let distance = (camera.center - camera.eye).length();
    writeln!(
        preview,
        "\x1b[0mA/D: rotar  W/S: inclinar  +/-: zoom  R: reiniciar  G: guardar  Q: salir"
    )
    .expect("Escribir en una cadena no puede fallar");
    write!(
        preview,
        "Cámara: ({:.2}, {:.2}, {:.2})  Distancia: {:.2}\n> ",
        camera.eye.x, camera.eye.y, camera.eye.z, distance
    )
    .expect("Escribir en una cadena no puede fallar");

    let stdout = io::stdout();
    let mut output = stdout.lock();
    output.write_all(preview.as_bytes())?;
    output.flush()
}

fn save_interactive_view(scene: &Scene, camera: &Camera) -> std::io::Result<()> {
    let mut framebuffer = Framebuffer::new(
        FINAL_RENDER_WIDTH,
        FINAL_RENDER_HEIGHT,
        Color::new(8, 10, 18),
    );
    render_refined(&mut framebuffer, camera, scene);
    write_png("interactive_view.png", &framebuffer)?;
    Ok(())
}

fn render_main_view(scene: &Scene) -> std::io::Result<()> {
    use std::f32::consts::PI;
    use std::time::Instant;

    let mut framebuffer = Framebuffer::new(
        FINAL_RENDER_WIDTH,
        FINAL_RENDER_HEIGHT,
        Color::new(8, 10, 18),
    );
    let camera = Camera::new(
        Vec3::new(11.5, 7.2, 12.5),
        Vec3::new(0.0, 1.45, -9.2),
        Vec3::new(0.0, 1.0, 0.0),
        PI / 3.5,
    );

    let render_start = Instant::now();
    render_refined(&mut framebuffer, &camera, scene);
    let render_time = render_start.elapsed();
    write_ppm("temple_cubes.ppm", &framebuffer)?;
    write_bmp("temple_cubes.bmp", &framebuffer)?;
    write_png("temple_cubes.png", &framebuffer)?;
    println!(
        "Render de {}x{} completado en {:.2?}",
        framebuffer.width, framebuffer.height, render_time
    );
    println!("Archivos generados: temple_cubes.ppm, temple_cubes.bmp y temple_cubes.png");

    Ok(())
}

fn render_animation(scene: &Scene) -> std::io::Result<()> {
    use std::f32::consts::{PI, TAU};

    const FRAME_COUNT: usize = 192;
    const OUTPUT_DIRECTORY: &str = "renders/animation";

    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let mut framebuffer = Framebuffer::new(800, 450, Color::new(8, 10, 18));

    for frame in 0..FRAME_COUNT {
        let progress = frame as f32 / (FRAME_COUNT - 1) as f32;
        let eased_progress = progress * progress * (3.0 - 2.0 * progress);
        let mut camera = Camera::new(
            Vec3::new(0.0, 8.4, 16.0),
            Vec3::new(0.0, 1.5, -9.5),
            Vec3::new(0.0, 1.0, 0.0),
            PI / 3.5,
        );
        let yaw = -0.78 + 1.56 * eased_progress;
        let pitch = 0.10 * (TAU * progress).sin();
        let zoom_curve = (PI * progress).sin();
        let zoom = 3.4 * zoom_curve * zoom_curve;

        camera.orbit(yaw, pitch);
        camera.zoom(zoom);
        render(&mut framebuffer, &camera, scene);

        let output_path = format!("{OUTPUT_DIRECTORY}/frame_{frame:03}.png");
        write_png(&output_path, &framebuffer)?;
        if frame % 12 == 0 || frame + 1 == FRAME_COUNT {
            println!("Cuadro {}/{FRAME_COUNT}: {output_path}", frame + 1);
        }
    }

    println!("Secuencia terminada en {OUTPUT_DIRECTORY}");
    Ok(())
}
