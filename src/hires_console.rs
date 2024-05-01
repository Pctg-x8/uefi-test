use core::fmt::Write;

struct FontDriver;
impl FontDriver {
    const BLOB: &'static [u8] = include_bytes!("../font.bin");
    pub const CHAR_W: u32 = 8;
    pub const CHAR_H: u32 = 12;

    fn get_row(c: char, y: u32) -> u8 {
        assert!(
            (0..256).contains(&(c as u32)),
            "out of ascii character cannot render"
        );

        Self::BLOB[(c as u32 * Self::CHAR_H + y) as usize]
    }

    pub fn render1(fb_base: &mut [[u8; 4]], fb_stride: u32, x: u32, y: u32, c: char) {
        for yo in 0..Self::CHAR_H {
            let r = Self::get_row(c, yo);
            for xo in 0..Self::CHAR_W {
                fb_base[((x + xo) + (y + yo) * fb_stride) as usize] = if (r >> xo) & 0x01 != 0 {
                    [255, 255, 255, 255]
                } else {
                    [0, 0, 0, 0]
                };
            }
        }
    }
}

pub struct HiResConsole {
    fb_base: &'static mut [[u8; 4]],
    fb_stride: u32,
    max_cols: u32,
    max_lines: u32,
    cursor_x: u32,
    cursor_y: u32,
}
impl HiResConsole {
    pub fn new(fb_base: &'static mut [[u8; 4]], fb_stride: u32, fb_height: u32) -> Self {
        Self {
            fb_base,
            fb_stride,
            max_cols: fb_stride / FontDriver::CHAR_W,
            max_lines: fb_height / FontDriver::CHAR_H,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn scroll_lines(&mut self, lines: u32) {
        let lines = lines.min(self.max_lines);
        if lines == self.max_lines {
            // clear operation
            self.fb_base.fill([0; 4]);
        } else {
            let gap = (lines * FontDriver::CHAR_H * self.fb_stride) as usize;
            unsafe {
                core::ptr::copy(
                    self.fb_base.as_mut_ptr().add(gap),
                    self.fb_base.as_mut_ptr(),
                    self.fb_base.len() - gap,
                );
            }
        }
    }

    pub fn newline(&mut self) {
        if self.cursor_y >= self.max_lines {
            self.scroll_lines(1);
        } else {
            self.cursor_y += 1;
        }
        self.cursor_x = 0;
    }

    pub fn write_char(&mut self, c: char) {
        FontDriver::render1(
            self.fb_base,
            self.fb_stride,
            self.cursor_x * FontDriver::CHAR_W,
            self.cursor_y * FontDriver::CHAR_H,
            c,
        );
        self.cursor_x += 1;
        if self.cursor_x >= self.max_cols {
            self.newline();
        }
    }
}
impl Write for HiResConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.newline();
            } else {
                self.write_char(c);
            }
        }

        Ok(())
    }
}
