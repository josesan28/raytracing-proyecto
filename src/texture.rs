use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::color::LinearColor;
use crate::math::{Vec2, Vec3};

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pixels: Vec<LinearColor>,
}

impl Texture {
    // Refuerza el detalle a escala media una sola vez, sin reducir resolución.
    // El promedio es periódico, igual que el muestreo de las superficies.
    pub fn enhance_detail(&mut self, amount: f32) {
        let radius = (self.width.min(self.height) / 32).clamp(1, 16);
        let diameter = radius * 2 + 1;
        let luminance: Vec<f32> = self
            .pixels
            .iter()
            .map(|p| p.r * 0.2126 + p.g * 0.7152 + p.b * 0.0722)
            .collect();
        let mut horizontal = vec![0.0; self.pixels.len()];
        for y in 0..self.height {
            let mut sum = 0.0;
            for offset in 0..diameter {
                let x =
                    (offset as isize - radius as isize).rem_euclid(self.width as isize) as usize;
                sum += luminance[y * self.width + x];
            }
            for x in 0..self.width {
                horizontal[y * self.width + x] = sum / diameter as f32;
                let outgoing =
                    (x as isize - radius as isize).rem_euclid(self.width as isize) as usize;
                let incoming = (x + radius + 1) % self.width;
                sum += luminance[y * self.width + incoming] - luminance[y * self.width + outgoing];
            }
        }
        for x in 0..self.width {
            let mut sum = 0.0;
            for offset in 0..diameter {
                let y =
                    (offset as isize - radius as isize).rem_euclid(self.height as isize) as usize;
                sum += horizontal[y * self.width + x];
            }
            for y in 0..self.height {
                let index = y * self.width + x;
                let detail = luminance[index] - sum / diameter as f32;
                // Ganancia acotada: conserva tonos y evita recortar luces/sombras.
                let gain = (1.0 + amount * detail / luminance[index].max(0.02)).clamp(0.65, 1.35);
                let pixel = self.pixels[index];
                let gain = gain.min(1.0 / pixel.r.max(pixel.g).max(pixel.b).max(0.001));
                self.pixels[index] = pixel.scale(gain);
                let outgoing =
                    (y as isize - radius as isize).rem_euclid(self.height as isize) as usize;
                let incoming = (y + radius + 1) % self.height;
                sum +=
                    horizontal[incoming * self.width + x] - horizontal[outgoing * self.width + x];
            }
        }
    }

    // Después de esta preparación, los canales guardan un vector tangente
    // firmado y normalizado, no un color RGB. Solo para mapas normales.
    pub fn prepare_normal_map(&mut self, strength: f32) {
        let strength = strength.clamp(0.0, 1.0);
        for pixel in &mut self.pixels {
            let normal = Vec3::new(
                (pixel.r * 2.0 - 1.0) * strength,
                (pixel.g * 2.0 - 1.0) * strength,
                pixel.b * 2.0 - 1.0,
            )
            .normalize();
            *pixel = LinearColor::new(normal.x, normal.y, normal.z);
        }
    }

    pub fn from_ppm(path: impl AsRef<Path>) -> io::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let mut tokens = contents
            .lines()
            .map(|line| line.split('#').next().unwrap_or_default())
            .flat_map(str::split_whitespace);

        if tokens.next() != Some("P3") {
            return Err(invalid_data("La textura debe usar el formato PPM P3"));
        }

        let width = parse_token::<usize>(&mut tokens, "ancho")?;
        let height = parse_token::<usize>(&mut tokens, "alto")?;
        let max_value = parse_token::<u32>(&mut tokens, "valor máximo")?;

        if width == 0 || height == 0 || max_value == 0 {
            return Err(invalid_data(
                "Las dimensiones y el valor máximo deben ser positivos",
            ));
        }

        let mut pixels = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            let r = parse_token::<u32>(&mut tokens, "canal rojo")?;
            let g = parse_token::<u32>(&mut tokens, "canal verde")?;
            let b = parse_token::<u32>(&mut tokens, "canal azul")?;

            if r > max_value || g > max_value || b > max_value {
                return Err(invalid_data("Un canal excede el valor máximo del PPM"));
            }

            let scale = max_value as f32;
            pixels.push(LinearColor::new(
                r as f32 / scale,
                g as f32 / scale,
                b as f32 / scale,
            ));
        }

        if tokens.next().is_some() {
            return Err(invalid_data(
                "La textura contiene más píxeles de los esperados",
            ));
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    // Interpola texeles vecinos, con continuidad en las texturas repetidas.
    // También admite los vectores firmados de los mapas normales preparados.
    pub fn sample_bilinear(&self, uv: Vec2) -> LinearColor {
        let x = uv.u.rem_euclid(1.0) * self.width as f32 - 0.5;
        let y = (1.0 - uv.v.rem_euclid(1.0)) * self.height as f32 - 0.5;
        let floor_x = x.floor();
        let floor_y = y.floor();
        // Las UV ya están envueltas: el único índice negativo posible es -1.
        // Evita cuatro divisiones enteras por muestra sin cambiar los texeles.
        let x0 = if floor_x < 0.0 {
            self.width - 1
        } else {
            floor_x as usize
        };
        let y0 = if floor_y < 0.0 {
            self.height - 1
        } else {
            floor_y as usize
        };
        let x1 = if x0 + 1 == self.width { 0 } else { x0 + 1 };
        let y1 = if y0 + 1 == self.height { 0 } else { y0 + 1 };
        let top =
            self.pixels[y0 * self.width + x0].lerp(self.pixels[y0 * self.width + x1], x - floor_x);
        let bottom =
            self.pixels[y1 * self.width + x0].lerp(self.pixels[y1 * self.width + x1], x - floor_x);
        top.lerp(bottom, y - floor_y)
    }

    pub fn sample_equirectangular(&self, direction: Vec3) -> LinearColor {
        use std::f32::consts::PI;

        let direction = direction.normalize();
        let u = 0.5 + direction.z.atan2(direction.x) / (2.0 * PI);
        let v = 0.5 - direction.y.clamp(-1.0, 1.0).asin() / PI;
        let x = u.rem_euclid(1.0) * self.width as f32 - 0.5;
        let y = v.clamp(0.0, 1.0) * (self.height - 1) as f32;
        let x0 = x.floor() as i32;
        let y0 = y.floor() as usize;
        let x1 = x0 + 1;
        let y1 = (y0 + 1).min(self.height - 1);
        let wrap_x = |value: i32| value.rem_euclid(self.width as i32) as usize;
        let top_left = self.pixels[y0 * self.width + wrap_x(x0)];
        let top_right = self.pixels[y0 * self.width + wrap_x(x1)];
        let bottom_left = self.pixels[y1 * self.width + wrap_x(x0)];
        let bottom_right = self.pixels[y1 * self.width + wrap_x(x1)];
        let horizontal = x - x.floor();
        let vertical = y - y.floor();

        top_left
            .lerp(top_right, horizontal)
            .lerp(bottom_left.lerp(bottom_right, horizontal), vertical)
    }
}

fn parse_token<'a, T: std::str::FromStr>(
    tokens: &mut impl Iterator<Item = &'a str>,
    name: &str,
) -> io::Result<T> {
    tokens
        .next()
        .ok_or_else(|| invalid_data(&format!("Falta el {name} del PPM")))?
        .parse()
        .map_err(|_| invalid_data(&format!("El {name} del PPM no es válido")))
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}
