use crate::color::Color;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    color_buffer: Vec<Color>,
    background_color: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, background_color: Color) -> Self {
        Self {
            width,
            height,
            color_buffer: vec![background_color; width * height],
            background_color,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer.fill(self.background_color);
    }

    pub fn pixels(&self) -> &[Color] {
        &self.color_buffer
    }

    pub fn pixels_mut(&mut self) -> &mut [Color] {
        &mut self.color_buffer
    }
}
