use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref CURRENT_KEY: Mutex<Option<char>> = Mutex::new(None);
}

pub fn start() {
    crate::WRITER.get().unwrap().lock().println("ENTER 5 KEYS");
    
    loop {
        x86_64::instructions::hlt();
        let key = input();
        crate::WRITER.get().unwrap().lock().print_char(key);
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