use core::arch::{asm, naked_asm};
use x86_64::registers::model_specific::{Efer, EferFlags, Msr};
use crate::processes::usermode::jump_to_user_mode;

// 1. נשנה את הפונקציה שתחזיר u64
#[no_mangle]
pub extern "C" fn syscall_handler(syscall_num: u64, str_ptr: u64, str_len: u64) -> u64 {
    unsafe {
        match syscall_num {
            0 => {
                let slice = core::slice::from_raw_parts(str_ptr as *const u8, str_len as usize);
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::WRITER.get().unwrap().lock().print(s);
                }
                0
            }
            1 => {
                // סיסקול 1: קבלת כל השורה שהוקלדה עד ה-Enter
                loop {
                    unsafe {
                        // אם לחצו אנטר (END_LINE הפך ל-true ב-set_key) וגם יש תוכן
                        if END_LINE && !FINALE_STR.is_empty() {
                            let user_buffer = str_ptr as *mut u8;
                            let mut bytes_written = 0;

                            // העתקת התווים מהבאפר של הקרנל לבאפר של ה-User Mode
                            for &ch in FINALE_STR.iter() {
                                *user_buffer.add(bytes_written) = ch as u8;
                                bytes_written += 1;
                            }

                            // קריטי: איפוס הבאפרים והדגלים של הקרנל לשורה הבאה!
                            KEY_LEN = 0;
                            FINALE_STR = &[];
                            END_LINE = false; // חייב לאפס גם את הדגל הזה!

                            // תיקון קריטי: שימוש ב-return מפורש כדי לצאת מה-syscall ולהחזיר את האורך ב-RAX
                            return bytes_written as u64;
                        }
                    }
                    // רמז למעבד שאנחנו בלולאת המתנה (מונע אופטימיזציות אגרסיביות וחניקה של הליבה)
                    core::hint::spin_loop();
                }
            }
            _ => 0,
        }
    }
}
#[unsafe(naked)]
pub extern "C" fn asm_syscall_handler() {
    unsafe {
        naked_asm!(
            "push rcx",
            "push r11",
            "push rbp",
            "push rbx",

            "mov rdi, rax",

            "call syscall_handler",

            "pop rbx",
            "pop rbp",
            "pop r11",
            "pop rcx",

            "sysretq"
        );
    }
}

pub fn init_syscalls() {
    unsafe {
        let mut efer = Efer::read();
        efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        Efer::write(efer);

        let mut lstar = Msr::new(0xC0000082);
        lstar.write(asm_syscall_handler as u64);

        let mut star = Msr::new(0xC0000081);
        let kernel_cs = 0x08u64;
        let user_cs_base = 0x1Bu64;

        let star_value = (user_cs_base << 48) | (kernel_cs << 32);
        star.write(star_value);
    }
}

static mut KEY_BUFFER: [char; 64] = ['\0'; 64];
static mut KEY_LEN: usize = 0;
static mut FINALE_STR: &[char] = &[];
static mut END_LINE: bool = false;

pub fn push_to_buffer(item: char) -> Result<(), &'static str> {
    unsafe {
        if KEY_LEN < KEY_BUFFER.len() {
            KEY_BUFFER[KEY_LEN] = item;
            KEY_LEN += 1;
            Ok(())
        } else {
            Err("Buffer overflow!")
        }
    }
}

pub fn pop_from_buffer() -> Option<char> {
    unsafe {
        if KEY_LEN > 0 {
            KEY_LEN -= 1;
            let removed_char = KEY_BUFFER[KEY_LEN];

            // (אופציונלי) איפוס התו לתו ריק כדי שהזיכרון יישאר נקי
            KEY_BUFFER[KEY_LEN] = '\0';

            Some(removed_char)
        } else {
            None // הבאפר כבר ריק
        }
    }
}

pub fn set_key(key: char) {
    if key == '\n'{
        unsafe {
            FINALE_STR = &KEY_BUFFER[..KEY_LEN];
            END_LINE = true;
        }
        return;
    }
    if key == '\x08'{
        crate::WRITER.get().unwrap().lock().backspace();
        pop_from_buffer();
        return;
    }
    let _ = push_to_buffer(key);
    crate::WRITER.get().unwrap().lock().print_char(key);
}

pub fn qemu_print_char(c: u8) {
    unsafe {
        asm!(
        "out dx, al",
        in("dx") 0xe9u16, // the debug i/o port
        in("al") c,       // the character byte to log
        );
    }
}

pub fn qemu_print_str(s: &str) {
    for byte in s.bytes() {
        unsafe {
            asm!(
            "out dx, al",
            in("dx") 0xe9u16, // the debug i/o port
            in("al") byte,       // the character byte to log
            );
        }
    }
}
