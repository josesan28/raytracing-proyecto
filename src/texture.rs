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

    pub fn sample_nearest(&self, uv: Vec2) -> LinearColor {
        let u = uv.u.rem_euclid(1.0);
        let v = uv.v.rem_euclid(1.0);
        let x = (u * self.width as f32).floor() as usize;
        let y = ((1.0 - v) * self.height as f32).floor() as usize;
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);

        self.pixels[y * self.width + x]
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
