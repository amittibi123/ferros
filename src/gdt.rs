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

#[repr(C, packed)]
struct TssDescriptor {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
    base_upper: u32,
    reserved: u32,
}

#[repr(C, packed)]
pub struct Tss {
    reserved0: u32,
    pub rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    iomap_base: u16,
}

impl Tss {
    const fn new() -> Self {
        Self {
            reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved1: 0,
            ist: [0; 7],
            reserved2: 0,
            reserved3: 0,
            iomap_base: size_of::<Tss>() as u16,
        }
    }
}

static mut TSS: Tss = Tss::new();
static mut KERNEL_STACK: [u8; 4096 * 4] = [0; 4096 * 4];

#[repr(C, packed)]
struct Gdt {
    null: GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_data: GdtEntry,
    user_code: GdtEntry,
    tss: TssDescriptor,
}

static mut GDT: Gdt = Gdt {
    null:        GdtEntry::new(0,      0,    0   ),
    kernel_code: GdtEntry::new(0xFFFF, 0x9A, 0xAF),
    kernel_data: GdtEntry::new(0xFFFF, 0x92, 0x00),
    user_data:   GdtEntry::new(0xFFFF, 0xF2, 0x00), // DPL=3, data
    user_code:   GdtEntry::new(0xFFFF, 0xFA, 0xAF), // DPL=3, code
    tss: TssDescriptor {
        limit_low: 0, base_low: 0, base_middle: 0,
        access: 0x89, granularity: 0, base_high: 0,
        base_upper: 0, reserved: 0,
    },
};

pub fn init() {
    unsafe {
        // הגדרת ה-stack של ring 0 שישמש כשקוד user מבצע interrupt/syscall
        let stack_top = &raw const KERNEL_STACK as u64 + (4096 * 4) as u64;
        TSS.rsp0 = stack_top;

        // מילוי כתובת ה-TSS בתוך ה-descriptor שלו
        let tss_addr = &raw const TSS as u64;
        let tss_limit = (size_of::<Tss>() - 1) as u16;

        GDT.tss.limit_low = tss_limit;
        GDT.tss.base_low = (tss_addr & 0xFFFF) as u16;
        GDT.tss.base_middle = ((tss_addr >> 16) & 0xFF) as u8;
        GDT.tss.base_high = ((tss_addr >> 24) & 0xFF) as u8;
        GDT.tss.base_upper = (tss_addr >> 32) as u32;

        let ptr = GdtPointer {
            size: (size_of::<Gdt>() - 1) as u16,
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
        "mov ax, 0x28", // selector של ה-TSS (offset 5*8 = 40 = 0x28)
        "ltr ax",
        in(reg) &ptr,
        out("rax") _,
        );
    }
}
