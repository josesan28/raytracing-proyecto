use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Clone, Copy, Debug, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            return self;
        }
        self / length
    }
}

use std::ops::{Add, Div, Mul, Sub};

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

trait RayIntersect {
    fn ray_intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> bool;
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> bool {
        // Vector from the ray origin to the sphere center.
        let oc = ray_origin - self.center;

        // Coefficients of: a*t² + b*t + c = 0.
        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * oc.dot(ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        discriminant >= 0.0
    }
}

enum Object {
    Sphere(Sphere),
}

impl RayIntersect for Object {
    fn ray_intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> bool {
        match self {
            Self::Sphere(sphere) => sphere.ray_intersect(ray_origin, ray_direction),
        }
    }
}

fn cast_ray(ray_origin: Vec3, ray_direction: Vec3, objects: &[Object]) -> [u8; 3] {
    for object in objects {
        if object.ray_intersect(ray_origin, ray_direction) {
            return [255, 255, 255];
        }
    }

    [0, 0, 0]
}

fn render(width: usize, height: usize, objects: &[Object], output_path: &str) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);

    // Encabezado PPM (Portable Pixmap).
    writeln!(output, "P3")?;
    writeln!(output, "{} {}", width, height)?;
    writeln!(output, "255")?;

    let aspect_ratio = width as f32 / height as f32;
    let ray_origin = Vec3::new(0.0, 0.0, 0.0);

    for y in 0..height {
        for x in 0..width {
            let screen_x = (2.0 * (x as f32 + 0.5)) / width as f32 - 1.0;
            let screen_y = 1.0 - (2.0 * (y as f32 + 0.5)) / height as f32;
            let screen_x = screen_x * aspect_ratio;

            let ray_direction = Vec3::new(screen_x, screen_y, -1.0).normalize();
            let color = cast_ray(ray_origin, ray_direction, objects);

            writeln!(output, "{} {} {}", color[0], color[1], color[2])?;
        }
    }

    output.flush()
}

fn main() -> std::io::Result<()> {
    let objects = vec![Object::Sphere(Sphere {
        center: Vec3::new(0.0, 0.0, -3.0),
        radius: 1.0,
    })];

    render(800, 600, &objects, "sphere.ppm")?;
    println!("Render terminado: sphere.ppm");
    Ok(())
}
