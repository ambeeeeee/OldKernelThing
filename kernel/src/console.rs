use console_vga::VgaConsole;
use driver_vga::Pixel;

use crate::graphics::graphics_ref;

pub fn setup_console() -> VgaConsole {
    let vga_graphics = graphics_ref();

    for x in 0..vga_graphics.width() {
        for y in 0..vga_graphics.height() {
            vga_graphics.set_pixel(
                x,
                y,
                Pixel {
                    red: 0,
                    green: 0,
                    blue: 0,
                },
            );
        }
    }

    VgaConsole::new(vga_graphics)
}
