#![no_std]
#![no_main]

use limine::request::FramebufferRequest;
use limine::BaseRevision;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        if let Some(fb) = fb_response.framebuffers().first() {
            for i in 0..200u64 {
                let offset = (i * fb.pitch + i * 4) as usize;
                unsafe {
                    let pixel = (fb.address() as *mut u32).add(offset / 4);
                    pixel.write_volatile(0x00FF00);
                }
            }
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
