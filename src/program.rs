use spin::Mutex;
use lazy_static::lazy_static;
use heapless::String;

use crate::{WRITER, program::shell::Dispatcher};
mod shell;

lazy_static! {
    static ref CURRENT_KEY: Mutex<Option<char>> = Mutex::new(None);
}

pub struct StringBuffer {
    data: [u8; 64], // גודל מקסימלי של 64 תווים
    len: usize,
}

impl StringBuffer {
    pub fn new() -> Self {
        Self {
            data: [0; 64],
            len: 0,
        }
    }

    pub fn add_char(&mut self, c: char) -> Result<(), &'static str> {
        if self.len < self.data.len() {
            // בקרנל בד"כ עובדים עם UTF-8, כאן אנחנו מניחים תווים פשוטים (ASCII)
            self.data[self.len] = c as u8;
            self.len += 1;
            Ok(())
        } else {
            Err("Buffer full")
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        // המרה בטוחה של המערך למחרוזת על בסיס האורך הנוכחי
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

pub fn start() {
    let mut s: String<32> = String::new();
    crate::WRITER.get().unwrap().lock().println("ENTER 5 KEYS");
    
    loop {
        x86_64::instructions::hlt();
        let key = input();
        match key {
            '\n' => {
                crate::WRITER.get().unwrap().lock().new_line();
                let final_str: &str = s.as_str();
                Dispatcher::dispatch_command(final_str);
                s.clear();
            },
            '\x08' => { // Backspace
                if s.len() > 0 {
                    s.pop(); // מוחק מה-String של heapless
                    crate::WRITER.get().unwrap().lock().backspace(); // מוחק מהמסך
                }
            },
            _ => {
                // הוספת תו רגיל
                if s.push(key).is_ok() {
                    crate::WRITER.get().unwrap().lock().print_char(key);
                }
            }
        }
        
    }
    
}

pub fn set_key(key: char) {
    *CURRENT_KEY.lock() = Some(key);
}

pub fn input() -> char {
    loop {
        let key = { *CURRENT_KEY.lock() }; // הסוגריים משחררים את ה-lock מיד
        if let Some(k) = key {
            *CURRENT_KEY.lock() = None;
            return k;
        }
        x86_64::instructions::hlt(); // חכה לפסיקה הבאה
    }
}