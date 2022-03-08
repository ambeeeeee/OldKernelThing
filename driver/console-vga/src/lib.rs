#![no_std]

extern crate alloc;

use core::fmt::Write;

use cansi::categorise_text;
use driver_vga::VGADriver;
use font8x8::{unicode::BasicFonts, UnicodeFonts};
use palette::{rgb::Rgb, Srgb, named};

pub trait RawConsoleDriver {
    /// Scrolls the console by the amount of lines provided
    fn scroll(&mut self, count: u32);

    /// Writes a character to the cursor
    fn write_char(&mut self, ch: char);

    fn write_formatted_char(&mut self, ch: FormattedChar);
}

impl Write for dyn RawConsoleDriver {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.bytes() {
            self.write_char(ch as _);
        }

        Ok(())
    }
}

/// When supported, allows writing ansi-formatted text to the console
pub trait AnsiConsoleDriver {
    fn write_str(&mut self, string: &str);
}

pub struct VgaConsole {
    driver: *const (dyn VGADriver + Send + Sync),
    row: usize,
    col: usize,
    height: usize,
    width: usize,
}

impl VgaConsole {
    pub fn new(driver: *const (dyn VGADriver + Send + Sync)) -> Self {
        let driver = unsafe { &*driver };
        let width = (driver.width() / 8) - 1;
        let height = (driver.height() / 8) - 1;

        Self {
            driver,
            row: 0,
            col: 0,
            width,
            height,
        }
    }

    fn driver(&self) -> &(dyn VGADriver + Send + Sync) {
        unsafe { &*self.driver }
    }
}

impl RawConsoleDriver for VgaConsole {
    fn scroll(&mut self, count: u32) {
        self.driver().shift_vertical(8 * count as usize);
    }

    fn write_char(&mut self, ch: char) {
        if ch == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            let y = self.row * 8;
            let x = self.col * 8;

            let glyph = BasicFonts::new().get(ch).unwrap_or([0xff; 8]);

            for glyph_row in 0..8 {
                let glyph_row_data = glyph[glyph_row];

                for glyph_col in 0..8 {
                    let mut pixel = driver_vga::Pixel {
                        red: 0,
                        green: 0,
                        blue: 0,
                    };

                    if (glyph_row_data & (1 << glyph_col)) != 0 {
                        pixel.red = 0xff;
                        pixel.green = 0xff;
                        pixel.blue = 0xff;
                    } else {
                        pixel.red = 0x00;
                        pixel.green = 0x00;
                        pixel.blue = 0x00;
                    }

                    self.driver().set_pixel(x + glyph_col, y + glyph_row, pixel);
                }
            }

            self.col += 1;
        }

        if self.col >= self.width {
            self.row += 1;
            self.col = 0;
        }

        if self.row >= self.height {
            self.scroll(1);
            self.row = self.height - 1;
        }
    }

    fn write_formatted_char(&mut self, ch: FormattedChar) {
        let foreground = ch.foreground.unwrap_or(Srgb::new(255, 255, 255));
        let background = ch.background.unwrap_or(Srgb::new( 0, 0, 0));
             

        if ch.ch == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            let y = self.row * 8;
            let x = self.col * 8;

            let glyph = BasicFonts::new().get(ch.ch).unwrap_or([0xff; 8]);

            for glyph_row in 0..8 {
                let glyph_row_data = glyph[glyph_row];

                for glyph_col in 0..8 {
                    let mut pixel = driver_vga::Pixel {
                        red: 0,
                        green: 0,
                        blue: 0,
                    };

                    if (glyph_row_data & (1 << glyph_col)) != 0 {
                        pixel.red = foreground.red;
                        pixel.green = foreground.green;
                        pixel.blue = foreground.blue;
                    } else {
                        pixel.red = background.red;
                        pixel.green = background.green;
                        pixel.blue = background.blue;
                    }

                    self.driver().set_pixel(x + glyph_col, y + glyph_row, pixel);
                }
            }

            self.col += 1;
        }

        if self.col >= self.width {
            self.row += 1;
            self.col = 0;
        }

        if self.row >= self.height {
            self.scroll(1);
            self.row = self.height - 1;
        }

    }
}

impl AnsiConsoleDriver for VgaConsole {
    fn write_str(&mut self, string: &str) {
        let categorised = categorise_text(string);

        for section in categorised {
            let foreground = {
                match section.fg_colour {
                    cansi::Color::Black => palette::named::BLACK,
                    cansi::Color::Red => named::RED,
                    cansi::Color::Green => named::GREEN,
                    cansi::Color::Yellow => named::YELLOW,
                    cansi::Color::Blue => named::BLUE,
                    cansi::Color::Magenta => named::MAGENTA,
                    cansi::Color::Cyan => named::CYAN,
                    cansi::Color::White => named::WHITE,
                    cansi::Color::BrightBlack => named::BLACK,
                    cansi::Color::BrightRed => named::RED,
                    cansi::Color::BrightGreen => named::LIMEGREEN,
                    cansi::Color::BrightYellow => named::YELLOWGREEN,
                    cansi::Color::BrightBlue => named::LIGHTBLUE,
                    cansi::Color::BrightMagenta => named::MAGENTA,
                    cansi::Color::BrightCyan => named::CYAN,
                    cansi::Color::BrightWhite => named::WHITE,
                }
            };

            let background = {
                match section.bg_colour {
                    cansi::Color::Black => palette::named::BLACK,
                    cansi::Color::Red => named::RED,
                    cansi::Color::Green => named::GREEN,
                    cansi::Color::Yellow => named::YELLOW,
                    cansi::Color::Blue => named::BLUE,
                    cansi::Color::Magenta => named::MAGENTA,
                    cansi::Color::Cyan => named::CYAN,
                    cansi::Color::White => named::WHITE,
                    cansi::Color::BrightBlack => named::BLACK,
                    cansi::Color::BrightRed => named::RED,
                    cansi::Color::BrightGreen => named::LIMEGREEN,
                    cansi::Color::BrightYellow => named::YELLOWGREEN,
                    cansi::Color::BrightBlue => named::LIGHTBLUE,
                    cansi::Color::BrightMagenta => named::MAGENTA,
                    cansi::Color::BrightCyan => named::CYAN,
                    cansi::Color::BrightWhite => named::WHITE,
                }
            };

            for ch in section.text.chars() {
                self.write_formatted_char(FormattedChar {
                    foreground: Some(foreground),
                    background: Some(background),
                    ch,
                })
            }
        }
    }
}

impl Write for VgaConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.bytes() {
            RawConsoleDriver::write_char(self, ch as _);
        }

        Ok(())
    }
}


pub struct FormattedChar {
    pub foreground: Option<palette::rgb::Srgb<u8>>,
    pub background: Option<palette::rgb::Srgb<u8>>,
    pub ch: char
}