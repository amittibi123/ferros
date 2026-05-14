#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod ata;
mod fat;
mod gdt;
mod interrupts;
mod program;
mod screen;

use limine::request::FramebufferRequest;
use limine::BaseRevision;
use spin::Mutex;
use spin::Once;

pub static WRITER: Once<Mutex<screen::Writer>> = Once::new();

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub fn handle_key(key: char) {
    program::set_key(key);
}

#[no_mangle]
extern "C" fn kmain() -> ! {
    gdt::init();
    interrupts::init(); // קודם כל — לפני הכל

    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        if let Some(fb) = fb_response.framebuffers().first() {
            WRITER.call_once(|| {
                Mutex::new(screen::Writer::new(
                    fb.address() as *mut u32 as u64,
                    fb.width,
                    fb.height,
                    fb.pitch,
                    0xFFFFFF,
                ))
            });
            // בדיקת ATA
            let mut buf = [0u8; 512];
            ata::read_sector(0, &mut buf);
            program::start();
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
