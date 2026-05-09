#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod gdt;
mod interrupts;
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
    if let Some(writer) = WRITER.get() {
        writer.lock().print_char(key);
    }
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

            WRITER.get().unwrap().lock().print("HELLO AMIT A\n");
            WRITER.get().unwrap().lock().print("TYPE SOMETHING:\n");
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
