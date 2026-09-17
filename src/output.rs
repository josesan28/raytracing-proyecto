use std::fs::File;
use std::io::{BufWriter, Write};

use crate::color::Color;

pub fn write_ppm(
    output_path: &str,
    width: usize,
    height: usize,
    pixels: &[Color],
) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut output = BufWriter::new(file);

    writeln!(output, "P3")?;
    writeln!(output, "{} {}", width, height)?;
    writeln!(output, "255")?;

    for color in pixels {
        writeln!(output, "{} {} {}", color.r, color.g, color.b)?;
    }

    output.flush()
}
