#![no_std]

extern crate alloc;

use bootloader::boot_info::{FrameBufferInfo, PixelFormat};
use driver_vga::{Pixel, VGADriver};

pub struct VgaEfi {
    buffer: *mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    format: PixelFormat,
    per_px: usize,
}

unsafe impl Send for VgaEfi {}
unsafe impl Sync for VgaEfi {}

impl VgaEfi {
    pub fn new(buffer: *mut [u8], info: FrameBufferInfo) -> Result<VgaEfi, ()> {
        Ok(Self {
            buffer,
            width: info.horizontal_resolution,
            height: info.vertical_resolution,
            stride: info.stride,
            format: info.pixel_format,
            per_px: info.bytes_per_pixel,
        })
    }
}

impl VGADriver for VgaEfi {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn shift_vertical(&self, _pixels: usize) {
        // STUB LETS GO
    }

    fn set_pixel(&self, x: usize, y: usize, value: Pixel) {
        assert!(y < self.height);
        assert!(x < self.width);

        let buffer = unsafe { &mut *self.buffer };

        // serial_println!("Writing to pixel {}", x + (y * self.height));

        let pixel_base = (x + (y * self.stride)) * self.per_px;

        // serial_println!("Base: {}", pixel_base);

        match self.format {
            PixelFormat::RGB => {
                buffer[pixel_base + 0] = value.red;
                buffer[pixel_base + 1] = value.blue;
                buffer[pixel_base + 2] = value.green;
            }
            PixelFormat::BGR => {
                buffer[pixel_base + 0] = value.blue;
                buffer[pixel_base + 1] = value.green;
                buffer[pixel_base + 2] = value.red;
            }
            PixelFormat::U8 => {
                todo!()
            }
            _ => unimplemented!(),
        }
    }
}
