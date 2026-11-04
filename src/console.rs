use crate::font::FONT8X8_BASIC;
use core::fmt;
pub struct Console {
    fb_ptr: *mut u32,
    width: usize,
    height: usize,
    x_pos: usize,
    y_pos: usize,
}

impl Console {
    pub const fn new() -> Self {
        Self {
            fb_ptr: core::ptr::null_mut(),
            width: 0,
            height: 0,
            x_pos: 0,
            y_pos: 0,
        }
    }

    pub unsafe fn init(&mut self, fb_ptr: *mut u32, width: usize, height: usize) {
        self.fb_ptr = fb_ptr;
        self.width = width;
        self.height = height;
    }
    fn write_pixel(&self, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        let pixel_offset = self.width * y + x;
        unsafe {
            *self.fb_ptr.add(pixel_offset) = 0xFFFF00FF;
        }
    }
    pub fn draw_char(&self, x: usize, y: usize, c: char, scale: usize) {
        let index = c as usize;
        if index >= FONT8X8_BASIC.len() {
            return;
        }
        let glyph = FONT8X8_BASIC[c as usize];
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (bits >> col) & 1 == 1 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            self.write_pixel(x + col * scale + dx, y + row * scale + dy);
                        }
                    }
                }
            }
        }
    }
    pub fn clear(&mut self) {
        if self.fb_ptr.is_null() {
            return;
        }

        let total_pixels = self.width * self.height;
        unsafe {
            for i in 0..total_pixels {
                *self.fb_ptr.add(i) = 0x00000000; // black
            }
        }

        self.x_pos = 0;
        self.y_pos = 0;
    }

    fn write_char(&mut self, c: char) {
        // 1. end of line next line
        if self.x_pos + 8 > self.width {
            self.x_pos = 0;
            self.y_pos += 16;
        }

        // 2. end of screen
        if self.y_pos + 16 > self.height {
            // self.y_pos = 0;
            self.clear();
        }

        if c == '\n' {
            self.x_pos = 0;
            self.y_pos += 16;
        } else {
            self.draw_char(self.x_pos, self.y_pos, c, 1);
            self.x_pos += 8;
        }
    }
}

// format output
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

static mut CONSOLE: Console = Console::new();

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        core::ptr::addr_of_mut!(CONSOLE)
            .as_mut()
            .unwrap()
            .write_fmt(args)
            .unwrap();
    }
}

pub unsafe fn init(fb_ptr: *mut u32, width: usize, height: usize) {
    unsafe {
        core::ptr::addr_of_mut!(CONSOLE)
            .as_mut()
            .unwrap()
            .init(fb_ptr, width, height);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::console::_print(core::format_args!("\n")));
    ($fmt:expr) => ($crate::console::_print(core::format_args!(concat!($fmt, "\n"))));
    ($fmt:expr, $($arg:tt)*) => ($crate::console::_print(core::format_args!(concat!($fmt, "\n"), $($arg)*)));
}
