use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::color::LinearColor;
use crate::math::Vec2;

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
