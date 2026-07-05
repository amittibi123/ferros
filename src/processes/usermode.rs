use core::arch::asm;

const USER_CODE_SELECTOR: u64 = 0x20 | 3; // 0x23
const USER_DATA_SELECTOR: u64 = 0x18 | 3; // 0x1B

pub unsafe fn jump_to_user_mode(entry_point: u64, user_stack: u64) -> ! {
    asm!(
    "mov ax, {data_sel:x}",
    "mov ds, ax",
    "mov es, ax",
    "mov fs, ax",
    "mov gs, ax",

    "push {data_sel}",   // SS
    "push {stack}",      // RSP
    "push 0x202",        // RFLAGS (interrupts enabled)
    "push {code_sel}",   // CS
    "push {entry}",      // RIP
    "iretq",

    data_sel = in(reg) USER_DATA_SELECTOR,
    code_sel = in(reg) USER_CODE_SELECTOR,
    stack = in(reg) user_stack,
    entry = in(reg) entry_point,
    options(noreturn)
    )
}