#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod ata;
mod fat;
mod gdt;
mod interrupts;
mod program;
mod screen;
mod processes;

use limine::request::FramebufferRequest;
use limine::BaseRevision;
use spin::Mutex;
use spin::Once;
use limine::request::MemmapRequest;
use spin::Mutex as SpinMutex;
use limine::request::HhdmRequest;
use crate::processes::{frame_allocator, loader, memory, syscall};
use crate::processes::syscall::{qemu_print_char, qemu_print_str};

pub static FRAME_ALLOCATOR: Once<SpinMutex<frame_allocator::FrameAllocator>> = Once::new();

#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

pub static WRITER: Once<Mutex<screen::Writer>> = Once::new();

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub fn handle_key(key: char) {
    syscall::set_key(key);
}

#[no_mangle]
extern "C" fn kmain() -> ! {
    gdt::init();
    interrupts::init();

    if let Some(mmap) = MEMORY_MAP_REQUEST.response() {
        FRAME_ALLOCATOR.call_once(|| {
            SpinMutex::new(frame_allocator::FrameAllocator::new(mmap))
        });
    }

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
            let mut buf = [0u8; 512];
            ata::read_sector(0, &mut buf);
        }
    }

    if let (Some(hhdm), Some(mmap)) = (HHDM_REQUEST.response(), MEMORY_MAP_REQUEST.response()) {
        let mut allocator = processes::frame_allocator::FrameAllocator::new(mmap);
        let space = processes::memory::create_process_page_table(hhdm.offset, &mut allocator);
        let code_frame = processes::memory::map_user_page(hhdm.offset, &space, &mut allocator, 0x500000);
        processes::memory::map_user_stack(hhdm.offset, &space, &mut allocator, 0x700000);
        processes::syscall::init_syscalls();
        unsafe {
            let dest = (hhdm.offset + code_frame.start_address().as_u64()) as *mut u8;
            core::ptr::copy_nonoverlapping(processes::loader::TEST_PROGRAM.as_ptr(), dest, processes::loader::TEST_PROGRAM.len());
            processes::memory::switch_to(&space);
            processes::usermode::jump_to_user_mode(0x500000, 0x700000);
        }
    }

    loop {}
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
