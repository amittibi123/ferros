use core::arch::asm;
use core::mem::size_of;

#[repr(C, packed)]
struct GdtPointer {
    size: u16,
    base: u64,
}

#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn new(limit: u16, access: u8, granularity: u8) -> Self {
        Self {
            limit_low: limit,
            base_low: 0,
            base_middle: 0,
            access,
            granularity,
            base_high: 0,
        }
    }
}

static mut GDT: [GdtEntry; 3] = [
    GdtEntry::new(0,      0,    0   ), // Null
    GdtEntry::new(0xFFFF, 0x9A, 0xAF), // Kernel Code 64-bit
    GdtEntry::new(0xFFFF, 0x92, 0x00), // Kernel Data
];

pub fn init() {
    unsafe {
        let ptr = GdtPointer {
            size: (size_of::<[GdtEntry; 3]>() - 1) as u16,
            base: &raw const GDT as u64,
        };
        asm!(
            "lgdt [{0}]",
            "push 0x08",
            "lea rax, [2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            in(reg) &ptr,
            out("rax") _,
        );
    }
}