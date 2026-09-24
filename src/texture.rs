use crate::color::LinearColor;

#[allow(dead_code)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<LinearColor>,
}

#[allow(dead_code)]
impl Texture {
    pub fn new(width: usize, height: usize, pixels: Vec<LinearColor>) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self {
            width,
            height,
            pixels,
        }
    }
}
