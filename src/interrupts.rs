use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use x86_64::instructions::port::Port;

use pic8259::ChainedPics;

use spin::Mutex;

use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 32;

pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt[33].set_handler_fn(keyboard_handler);

        idt
    };
}

pub fn init() {
    unsafe {
        PICS.lock().initialize();
    }
    IDT.load();
    unsafe {
        // בטל מסכה של IRQ0 (timer) ו-IRQ1 (keyboard) בלבד
        // שאר ה-IRQs נשארים מסוכמים
        use x86_64::instructions::port::Port;
        let mut master_data: Port<u8> = Port::new(0x21);
        let mut slave_data: Port<u8> = Port::new(0xA1);
        master_data.write(0b11111101); // רק IRQ1 פתוח, IRQ0 (timer) סגור
        slave_data.write(0b11111111);

        x86_64::instructions::interrupts::enable();
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    // ציור פיקסל ירוק כדי לראות אם הגענו לכאן
    if let Some(writer) = crate::WRITER.get() {
        writer.lock().draw_pixel(0, 0, 0x00FF00);
    }

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Some(key) = translate_scancode(scancode) {
        crate::handle_key(key);
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(33);
    }
}

fn translate_scancode(scancode: u8) -> Option<char> {
    match scancode {
        0x1E => Some('A'),
        0x30 => Some('B'),
        0x2E => Some('C'),

        0x20 => Some('D'),
        0x12 => Some('E'),
        0x21 => Some('F'),

        0x22 => Some('G'),
        0x23 => Some('H'),
        0x17 => Some('I'),

        0x24 => Some('J'),
        0x25 => Some('K'),
        0x26 => Some('L'),

        0x32 => Some('M'),
        0x31 => Some('N'),
        0x18 => Some('O'),

        0x19 => Some('P'),
        0x10 => Some('Q'),
        0x13 => Some('R'),

        0x1F => Some('S'),
        0x14 => Some('T'),
        0x16 => Some('U'),

        0x2F => Some('V'),
        0x11 => Some('W'),
        0x2D => Some('X'),

        0x15 => Some('Y'),
        0x2C => Some('Z'),
        0x1C => Some('\n'),
        0x39 => Some(' '),
        0x0E => Some('\x08'),
        0x34 => Some('.'),
        _ => None,
    }
}
