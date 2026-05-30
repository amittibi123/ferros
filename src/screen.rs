use crate::WRITER;
use limine::framebuffer::Framebuffer;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;
const FONT: [[u8; 8]; 26] = [
    [
        0b00011000, 0b00100100, 0b01000010, 0b01111110, 0b01000010, 0b01000010, 0b01000010,
        0b00000000,
    ],
    [
        0b01111100, 0b01000010, 0b01000010, 0b01111100, 0b01000010, 0b01000010, 0b01111100,
        0b00000000,
    ],
    [
        0b00111100, 0b01000010, 0b01000000, 0b01000000, 0b01000000, 0b01000010, 0b00111100,
        0b00000000,
    ],
    [
        0b01111000, 0b01000100, 0b01000010, 0b01000010, 0b01000010, 0b01000100, 0b01111000,
        0b00000000,
    ],
    [
        0b01111110, 0b01000000, 0b01000000, 0b01111100, 0b01000000, 0b01000000, 0b01111110,
        0b00000000,
    ],
    [
        0b01111110, 0b01000000, 0b01000000, 0b01111100, 0b01000000, 0b01000000, 0b01000000,
        0b00000000,
    ],
    [
        0b00111100, 0b01000010, 0b01000000, 0b01001110, 0b01000010, 0b01000010, 0b00111100,
        0b00000000,
    ],
    [
        0b01000010, 0b01000010, 0b01000010, 0b01111110, 0b01000010, 0b01000010, 0b01000010,
        0b00000000,
    ],
    [
        0b00011100, 0b00001000, 0b00001000, 0b00001000, 0b00001000, 0b00001000, 0b00011100,
        0b00000000,
    ],
    [
        0b00000110, 0b00000010, 0b00000010, 0b00000010, 0b00000010, 0b01000010, 0b00111100,
        0b00000000,
    ],
    [
        0b01000100, 0b01001000, 0b01010000, 0b01100000, 0b01010000, 0b01001000, 0b01000100,
        0b00000000,
    ],
    [
        0b01000000, 0b01000000, 0b01000000, 0b01000000, 0b01000000, 0b01000000, 0b01111110,
        0b00000000,
    ],
    [
        0b01000010, 0b01100110, 0b01011010, 0b01000010, 0b01000010, 0b01000010, 0b01000010,
        0b00000000,
    ],
    [
        0b01000010, 0b01100010, 0b01010010, 0b01001010, 0b01000110, 0b01000010, 0b01000010,
        0b00000000,
    ],
    [
        0b00111100, 0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b00111100,
        0b00000000,
    ],
    [
        0b01111100, 0b01000010, 0b01000010, 0b01111100, 0b01000000, 0b01000000, 0b01000000,
        0b00000000,
    ],
    [
        0b00111100, 0b01000010, 0b01000010, 0b01000010, 0b01001010, 0b01000100, 0b00111010,
        0b00000000,
    ],
    [
        0b01111100, 0b01000010, 0b01000010, 0b01111100, 0b01010000, 0b01001000, 0b01000100,
        0b00000000,
    ],
    [
        0b00111110, 0b01000000, 0b01000000, 0b00111100, 0b00000010, 0b00000010, 0b01111100,
        0b00000000,
    ],
    [
        0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
        0b00000000,
    ],
    [
        0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b00111100,
        0b00000000,
    ],
    [
        0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b00100100, 0b00011000,
        0b00000000,
    ],
    [
        0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b01011010, 0b01100110, 0b01000010,
        0b00000000,
    ],
    [
        0b01000010, 0b00100100, 0b00011000, 0b00011000, 0b00011000, 0b00100100, 0b01000010,
        0b00000000,
    ],
    [
        0b01000010, 0b01000010, 0b01000010, 0b00111100, 0b00011000, 0b00011000, 0b00011000,
        0b00000000,
    ],
    [
        0b01111110, 0b00000100, 0b00001000, 0b00011100, 0b00100000, 0b01000000, 0b01111110,
        0b00000000,
    ],
];
const FONT_DOT: [u8; 8] = [
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000,
];

const FONT_SLASH: [u8; 8] = [
    0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000, 0b00000000,
];

const FONT_GT: [u8; 8] = [
    0b01100000, 0b00011000, 0b00000110, 0b00000110, 0b00011000, 0b01100000, 0b00000000, 0b00000000,
];

const FONT_LT: [u8; 8] = [
    0b00000110, 0b00011000, 0b01100000, 0b01100000, 0b00011000, 0b00000110, 0b00000000, 0b00000000,
];
pub struct Writer {
    x: u64,
    y: u64,
    color: u32,
    addr: u64,
    width: u64,
    height: u64,
    pitch: u64,
}

unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

impl Writer {
    pub fn new(addr: u64, width: u64, height: u64, pitch: u64, color: u32) -> Self {
        Self {
            x: 20,
            y: 20,
            color,
            addr,
            width,
            height,
            pitch,
        }
    }

    pub fn draw_pixel(&self, x: u64, y: u64, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = (y * self.pitch + x * 4) as usize;
        unsafe {
            let pixel = (self.addr as *mut u32).add(offset / 4);
            pixel.write_volatile(color);
        }
    }

    pub fn backspace(&mut self) {
        if self.x > 20 {
            // ודא שאנחנו לא מוחקים את ה-Prompt (התחלת השורה)
            self.x -= 10;
            // צייר מלבן ריק בצבע הרקע (נניח שחור 0x000000)
            for row in 0..8 {
                for col in 0..8 {
                    self.draw_pixel(self.x + col, self.y + row, 0x000000);
                }
            }
        }
    }

    pub fn clear_screen(&mut self) {
        // אנחנו רצים על כל השורות (height)
        for y in 0..self.height {
            // ועל כל העמודות (width)
            for x in 0..self.width {
                self.draw_pixel(x, y, 0x000000);
            }
        }
        // אחרי הניקוי, נחזיר את הסמן להתחלה
        self.x = 20;
        self.y = 20;
    }

    pub fn print_char(&mut self, mut c: char) {
        c = c.to_ascii_uppercase();
        if c == '\n' {
            self.new_line();
            return;
        }
        if c == ' ' {
            if self.x + 10 >= self.width {
                self.new_line();
                return;
            }
            self.x += 10;
            return;
        }
        if self.x + 10 >= self.width {
            self.new_line();
        }

        // אותיות אנגליות
        if c >= 'A' && c <= 'Z' {
            let index = (c as usize) - ('A' as usize);
            let glyph = FONT[index];
            for row in 0..8usize {
                let row_data = glyph[row];
                for col in 0..8usize {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        self.draw_pixel(self.x + col as u64, self.y + row as u64, self.color);
                    }
                }
            }
        }

        // נקודה
        if c == '.' {
            let glyph = FONT_DOT;
            for row in 0..8usize {
                let row_data = glyph[row];
                for col in 0..8usize {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        self.draw_pixel(self.x + col as u64, self.y + row as u64, self.color);
                    }
                }
            }
        }

        // 🌟 תמיכה בנטוי ימני /
        if c == '/' {
            let glyph = FONT_SLASH;
            for row in 0..8usize {
                let row_data = glyph[row];
                for col in 0..8usize {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        self.draw_pixel(self.x + col as u64, self.y + row as u64, self.color);
                    }
                }
            }
        }

        // 🌟 תמיכה בסימן קטן-מ- >
        if c == '>' {
            let glyph = FONT_GT;
            for row in 0..8usize {
                let row_data = glyph[row];
                for col in 0..8usize {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        self.draw_pixel(self.x + col as u64, self.y + row as u64, self.color);
                    }
                }
            }
        }

        // 🌟 תמיכה בסימן גדול-מ- <
        if c == '<' {
            let glyph = FONT_LT;
            for row in 0..8usize {
                let row_data = glyph[row];
                for col in 0..8usize {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        self.draw_pixel(self.x + col as u64, self.y + row as u64, self.color);
                    }
                }
            }
        }

        self.x += 10;
    }
    pub fn print(&mut self, text: &str) {
        for c in text.chars() {
            self.print_char(c); // תן ל-print_char לטפל גם ב-n\ וגם ברווח
        }
    }

    pub fn println(&mut self, text: &str) {
        self.print(text);
        self.new_line();
    }

    pub fn new_line(&mut self) {
        self.x = 20;
        if self.y + 24 >= self.height {
            self.scroll();
        } else {
            self.y += 12;
        }
    }
    fn scroll(&mut self) {
        unsafe {
            let base = self.addr as *mut u8;
            let row_bytes = (self.pitch) as usize;
            let scroll_bytes = 12 * row_bytes;
            let total_bytes = self.height as usize * row_bytes;

            // העתק הכל בבת אחת
            core::ptr::copy(base.add(scroll_bytes), base, total_bytes - scroll_bytes);

            // נקה שורה אחרונה
            core::ptr::write_bytes(base.add(total_bytes - scroll_bytes), 0, scroll_bytes);
        }
    }
}
