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
