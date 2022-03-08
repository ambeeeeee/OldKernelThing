use bootloader::{boot_info::MemoryRegionKind, BootInfo};
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

/// Allocates usable frames from the bootloader's memory map.
pub struct FrameAllocatorBootInfo {
    /// The memory map passed to the entry point of the kernel at boot, used to
    /// allocate pages
    memory_map: &'static BootInfo,
    next: usize,
}

impl FrameAllocatorBootInfo {
    /// Creates a FrameAllocator from a `MemoryMap` from the bootloader
    ///
    /// # Safety
    /// The caller must guarantee that the passed memory map is valid. Mainly,
    /// that all frames marked as `USABLE` are indeed unused.
    pub unsafe fn init(boot_info_memory_map: &'static BootInfo) -> Self {
        Self {
            memory_map: boot_info_memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = &self.memory_map.memory_regions;
        let usable_regions = regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorBootInfo {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
