use crate::color::Color;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    color_buffer: Vec<Color>,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, background_color: Color) -> Self {
        Self {
            width,
            height,
            color_buffer: vec![background_color; width * height],
            background_color,
            current_color: Color::new(255, 255, 255),
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer.fill(self.background_color);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.color_buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn pixels(&self) -> &[Color] {
        &self.color_buffer
    }
}
