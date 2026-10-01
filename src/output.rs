use std::fs::File;
use std::io::{self, BufWriter, Write};

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

pub fn write_png(output_path: &str, framebuffer: &Framebuffer) -> io::Result<()> {
    let width = u32::try_from(framebuffer.width).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "El ancho excede el formato PNG",
        )
    })?;
    let height = u32::try_from(framebuffer.height).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "El alto excede el formato PNG")
    })?;

    let row_size = framebuffer
        .width
        .checked_mul(3)
        .and_then(|size| size.checked_add(1))
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "La imagen es demasiado grande")
        })?;
    let capacity = row_size.checked_mul(framebuffer.height).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "La imagen es demasiado grande")
    })?;
    let mut scanlines = Vec::with_capacity(capacity);

    for row in framebuffer.pixels().chunks(framebuffer.width) {
        scanlines.push(0); // Filtro PNG: ninguno.
        for color in row {
            scanlines.extend_from_slice(&[color.r, color.g, color.b]);
        }
    }

    let compressed = zlib_uncompressed(&scanlines);
    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);
    output.write_all(b"\x89PNG\r\n\x1a\n")?;

    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]); // RGB de 8 bits, sin entrelazado.
    write_png_chunk(&mut output, b"IHDR", &header)?;
    write_png_chunk(&mut output, b"IDAT", &compressed)?;
    write_png_chunk(&mut output, b"IEND", &[])?;
    output.flush()
}

fn zlib_uncompressed(data: &[u8]) -> Vec<u8> {
    let block_count = data.len().div_ceil(u16::MAX as usize);
    let mut output = Vec::with_capacity(data.len() + block_count * 5 + 6);
    output.extend_from_slice(&[0x78, 0x01]);

    for (index, block) in data.chunks(u16::MAX as usize).enumerate() {
        let is_final = index + 1 == block_count;
        output.push(u8::from(is_final));
        let length = block.len() as u16;
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(&(!length).to_le_bytes());
        output.extend_from_slice(block);
    }

    output.extend_from_slice(&adler32(data).to_be_bytes());
    output
}

fn write_png_chunk(output: &mut impl Write, kind: &[u8; 4], data: &[u8]) -> io::Result<()> {
    let length = u32::try_from(data.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "El bloque PNG es demasiado grande",
        )
    })?;
    output.write_all(&length.to_be_bytes())?;
    output.write_all(kind)?;
    output.write_all(data)?;

    let mut checksum_data = Vec::with_capacity(kind.len() + data.len());
    checksum_data.extend_from_slice(kind);
    checksum_data.extend_from_slice(data);
    output.write_all(&crc32(&checksum_data).to_be_bytes())
}

fn adler32(data: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let mut first = 1_u32;
    let mut second = 0_u32;

    for &byte in data {
        first = (first + u32::from(byte)) % MODULUS;
        second = (second + first) % MODULUS;
    }

    (second << 16) | first
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
