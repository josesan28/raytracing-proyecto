use std::fs::File;
use std::io::{BufWriter, Write};

use crate::color::Color;

pub fn write_bmp(
    output_path: &str,
    width: usize,
    height: usize,
    pixels: &[Color],
) -> std::io::Result<()> {
    let row_padding = (4 - (width * 3) % 4) % 4;
    let row_size = width * 3 + row_padding;
    let image_size = row_size * height;
    let file_size = 54 + image_size;

    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);

    output.write_all(b"BM")?;
    output.write_all(&(file_size as u32).to_le_bytes())?;
    output.write_all(&[0; 4])?;
    output.write_all(&54_u32.to_le_bytes())?;

    output.write_all(&40_u32.to_le_bytes())?;
    output.write_all(&(width as i32).to_le_bytes())?;
    output.write_all(&(height as i32).to_le_bytes())?;
    output.write_all(&1_u16.to_le_bytes())?;
    output.write_all(&24_u16.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;
    output.write_all(&(image_size as u32).to_le_bytes())?;
    output.write_all(&2835_i32.to_le_bytes())?;
    output.write_all(&2835_i32.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;
    output.write_all(&0_u32.to_le_bytes())?;

    let padding = vec![0; row_padding];

    for y in (0..height).rev() {
        for x in 0..width {
            let [red, green, blue] = pixels[y * width + x];
            output.write_all(&[blue, green, red])?;
        }
        output.write_all(&padding)?;
    }

    output.flush()
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;

    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }

    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let mut first = 1_u32;
    let mut second = 0_u32;

    for &byte in bytes {
        first = (first + byte as u32) % MODULUS;
        second = (second + first) % MODULUS;
    }

    (second << 16) | first
}

fn zlib_uncompressed(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() + bytes.len() / 65_535 * 5 + 6);
    output.extend_from_slice(&[0x78, 0x01]);

    let mut offset = 0;

    while offset < bytes.len() {
        let block_length = (bytes.len() - offset).min(65_535);
        let is_final = offset + block_length == bytes.len();
        let length = block_length as u16;

        output.push(if is_final { 0x01 } else { 0x00 });
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(&(!length).to_le_bytes());
        output.extend_from_slice(&bytes[offset..offset + block_length]);

        offset += block_length;
    }

    output.extend_from_slice(&adler32(bytes).to_be_bytes());
    output
}

fn write_png_chunk<W: Write>(
    output: &mut W,
    chunk_type: &[u8; 4],
    data: &[u8],
) -> std::io::Result<()> {
    output.write_all(&(data.len() as u32).to_be_bytes())?;
    output.write_all(chunk_type)?;
    output.write_all(data)?;

    let mut crc_input = Vec::with_capacity(chunk_type.len() + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    output.write_all(&crc32(&crc_input).to_be_bytes())
}

pub fn write_png(
    output_path: &str,
    width: usize,
    height: usize,
    pixels: &[Color],
) -> std::io::Result<()> {
    let mut image_data = Vec::with_capacity((width * 3 + 1) * height);

    for y in 0..height {
        image_data.push(0);

        for x in 0..width {
            image_data.extend_from_slice(&pixels[y * width + x]);
        }
    }

    let compressed_data = zlib_uncompressed(&image_data);
    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);

    output.write_all(&[137, 80, 78, 71, 13, 10, 26, 10])?;

    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);

    write_png_chunk(&mut output, b"IHDR", &header)?;
    write_png_chunk(&mut output, b"IDAT", &compressed_data)?;
    write_png_chunk(&mut output, b"IEND", &[])?;
    output.flush()
}
