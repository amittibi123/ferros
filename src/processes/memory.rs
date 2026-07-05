use x86_64::structures::paging::{PageTable, PhysFrame, FrameAllocator as X86FrameAllocator, Size4KiB, Mapper, Page, PageTableFlags, OffsetPageTable};
use x86_64::{PhysAddr, VirtAddr};

pub unsafe fn init(hhdm_offset: u64) -> OffsetPageTable<'static> {
    let phys_offset = VirtAddr::new(hhdm_offset);
    let level_4_table = active_level_4_table(phys_offset);
    OffsetPageTable::new(level_4_table, phys_offset)
}

unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}


pub struct ProcessAddressSpace {
    pub level_4_frame: PhysFrame,
}

pub fn create_process_page_table(
    hhdm_offset: u64,
    frame_allocator: &mut impl X86FrameAllocator<Size4KiB>,
) -> ProcessAddressSpace {
    let new_frame = frame_allocator
        .allocate_frame()
        .expect("no frames left for new page table");

    let phys = new_frame.start_address();
    let virt = VirtAddr::new(hhdm_offset + phys.as_u64());
    let new_table: &mut PageTable = unsafe { &mut *virt.as_mut_ptr() };
    new_table.zero();

    // מעתיקים את חצי הקרנל (entries 256-511) מה-page table הנוכחי
    // כדי שאחרי מעבר CR3 הקרנל עדיין יעבוד
    let current_frame = x86_64::registers::control::Cr3::read().0;
    let current_phys = current_frame.start_address();
    let current_virt = VirtAddr::new(hhdm_offset + current_phys.as_u64());
    let current_table: &PageTable = unsafe { &*current_virt.as_ptr() };

    for i in 256..512 {
        new_table[i] = current_table[i].clone();
    }

    ProcessAddressSpace { level_4_frame: new_frame }
}

pub unsafe fn switch_to(space: &ProcessAddressSpace) {
    use x86_64::registers::control::{Cr3, Cr3Flags};
    Cr3::write(space.level_4_frame, Cr3Flags::empty());
}

pub fn map_user_page(
    hhdm_offset: u64,
    space: &ProcessAddressSpace,
    frame_allocator: &mut impl X86FrameAllocator<Size4KiB>,
    virt_addr: u64,
) -> PhysFrame {
    let phys = space.level_4_frame.start_address();
    let table_virt = VirtAddr::new(hhdm_offset + phys.as_u64());
    let table: &mut PageTable = unsafe { &mut *table_virt.as_mut_ptr() };
    let mut mapper = unsafe { OffsetPageTable::new(table, VirtAddr::new(hhdm_offset)) };

    let page = Page::containing_address(VirtAddr::new(virt_addr));
    let code_frame = frame_allocator.allocate_frame().expect("no frame for code");
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    unsafe {
        mapper.map_to(page, code_frame, flags, frame_allocator)
            .expect("map_to failed")
            .flush();
    }

    code_frame
}

pub fn map_user_stack(
    hhdm_offset: u64,
    space: &ProcessAddressSpace,
    frame_allocator: &mut impl X86FrameAllocator<Size4KiB>,
    stack_top_virt: u64,
) -> u64 {
    let phys = space.level_4_frame.start_address();
    let table_virt = VirtAddr::new(hhdm_offset + phys.as_u64());
    let table: &mut PageTable = unsafe { &mut *table_virt.as_mut_ptr() };
    let mut mapper = unsafe { OffsetPageTable::new(table, VirtAddr::new(hhdm_offset)) };

    let stack_page = Page::containing_address(VirtAddr::new(stack_top_virt - 0x1000));
    let stack_frame = frame_allocator.allocate_frame().expect("no frame for stack");
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    unsafe {
        mapper.map_to(stack_page, stack_frame, flags, frame_allocator)
            .expect("map_to failed")
            .flush();
    }

    stack_top_virt
}