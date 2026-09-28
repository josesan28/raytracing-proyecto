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
mod window;

use std::fs;
use std::io::{self, BufRead, Write};

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use math::Vec3;
use output::{write_bmp, write_ppm};
use renderer::render;
use scene::{Scene, temple_scene};

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
        Vec3::new(3.65, 2.55, 2.15),
        Vec3::new(0.0, 0.75, -5.7),
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
    let mut framebuffer = Framebuffer::new(800, 450, Color::new(8, 10, 18));
    render(&mut framebuffer, camera, scene);
    write_bmp("interactive_view.bmp", &framebuffer)?;
    Ok(())
}

fn render_main_view(scene: &Scene) -> std::io::Result<()> {
    use std::f32::consts::PI;

    let mut framebuffer = Framebuffer::new(800, 450, Color::new(8, 10, 18));
    let camera = Camera::new(
        Vec3::new(3.65, 2.55, 2.15),
        Vec3::new(0.0, 0.75, -5.7),
        Vec3::new(0.0, 1.0, 0.0),
        PI / 3.5,
    );

    render(&mut framebuffer, &camera, scene);
    write_ppm("temple_cubes.ppm", &framebuffer)?;
    write_bmp("temple_cubes.bmp", &framebuffer)?;
    println!("Render terminado: temple_cubes.ppm y temple_cubes.bmp");

    Ok(())
}

fn render_animation(scene: &Scene) -> std::io::Result<()> {
    use std::f32::consts::{PI, TAU};

    const FRAME_COUNT: usize = 24;
    const OUTPUT_DIRECTORY: &str = "renders/animation";

    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let mut framebuffer = Framebuffer::new(400, 225, Color::new(8, 10, 18));

    for frame in 0..FRAME_COUNT {
        let progress = frame as f32 / (FRAME_COUNT - 1) as f32;
        let mut camera = Camera::new(
            Vec3::new(0.0, 2.8, 5.4),
            Vec3::new(0.0, 0.8, -5.8),
            Vec3::new(0.0, 1.0, 0.0),
            PI / 3.5,
        );
        let yaw = -0.72 + 1.44 * progress;
        let pitch = 0.08 * (TAU * progress).sin();
        let zoom = 2.6 * (PI * progress).sin();

        camera.orbit(yaw, pitch);
        camera.zoom(zoom);
        render(&mut framebuffer, &camera, scene);

        let output_path = format!("{OUTPUT_DIRECTORY}/frame_{frame:03}.bmp");
        write_bmp(&output_path, &framebuffer)?;
        println!("Cuadro {}/{FRAME_COUNT}: {output_path}", frame + 1);
    }

    println!("Secuencia terminada en {OUTPUT_DIRECTORY}");
    Ok(())
}
