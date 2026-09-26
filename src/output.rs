use std::fs::File;
use std::io::{BufWriter, Write};

use crate::framebuffer::Framebuffer;

pub fn write_ppm(output_path: &str, framebuffer: &Framebuffer) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);

    writeln!(output, "P3")?;
    writeln!(output, "{} {}", framebuffer.width, framebuffer.height)?;
    writeln!(output, "255")?;

    for color in framebuffer.pixels() {
        writeln!(output, "{} {} {}", color.r, color.g, color.b)?;
    }

    output.flush()
}

pub fn write_bmp(output_path: &str, framebuffer: &Framebuffer) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);
    let row_size = framebuffer.width * 3;
    let padding = (4 - row_size % 4) % 4;
    let pixel_data_size = (row_size + padding) * framebuffer.height;
    let file_size = 54 + pixel_data_size;

    output.write_all(b"BM")?;
    output.write_all(&(file_size as u32).to_le_bytes())?;
    output.write_all(&[0; 4])?;
    output.write_all(&54_u32.to_le_bytes())?;
    output.write_all(&40_u32.to_le_bytes())?;
    output.write_all(&(framebuffer.width as i32).to_le_bytes())?;
    output.write_all(&(framebuffer.height as i32).to_le_bytes())?;
    output.write_all(&1_u16.to_le_bytes())?;
    output.write_all(&24_u16.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;
    output.write_all(&(pixel_data_size as u32).to_le_bytes())?;
    output.write_all(&[0; 16])?;

    for y in (0..framebuffer.height).rev() {
        let row_start = y * framebuffer.width;
        for color in &framebuffer.pixels()[row_start..row_start + framebuffer.width] {
            output.write_all(&[color.b, color.g, color.r])?;
        }
        output.write_all(&[0_u8; 3][..padding])?;
    }

    output.flush()
}
