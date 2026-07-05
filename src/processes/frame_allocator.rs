use limine::memmap::{Entry, MEMMAP_USABLE};
use limine::request::MemmapResponse;
use x86_64::structures::paging::{FrameAllocator as X86FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub struct FrameAllocator {
    regions: [(u64, u64); 32],
    region_count: usize,
    cursor_region: usize,
    cursor_addr: u64,
}

impl FrameAllocator {
    pub fn new(mmap: &MemmapResponse) -> Self {
        let mut regions = [(0u64, 0u64); 32];
        let mut count = 0;

        for entry in mmap.entries() {
            if entry.type_ == MEMMAP_USABLE && count < 32 {
                regions[count] = (entry.base, entry.base + entry.length);
                count += 1;
            }
        }

        let first_addr = if count > 0 { regions[0].0 } else { 0 };

        Self {
            regions,
            region_count: count,
            cursor_region: 0,
            cursor_addr: first_addr,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<u64> {
        while self.cursor_region < self.region_count {
            let (_, end) = self.regions[self.cursor_region];
            if self.cursor_addr < end {
                let frame = self.cursor_addr;
                self.cursor_addr += 4096;
                return Some(frame);
            }
            self.cursor_region += 1;
            if self.cursor_region < self.region_count {
                self.cursor_addr = self.regions[self.cursor_region].0;
            }
        }
        None
    }
}

unsafe impl X86FrameAllocator<Size4KiB> for FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = self.allocate_frame()?;
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
