#![no_std]

pub trait VGADriver {
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn shift_vertical(&self, pixels: usize);

    fn set_pixel(&self, x: usize, y: usize, value: Pixel);
}

pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
