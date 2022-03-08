use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

#[derive(Debug, Clone, Copy)]
pub struct StackBounds {
    start: VirtAddr,
    end: VirtAddr,
}

impl StackBounds {
    pub fn start(&self) -> VirtAddr {
        self.start
    }

    pub fn end(&self) -> VirtAddr {
        self.end
    }
}

pub fn alloc_stack(
    size_in_pages: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<StackBounds, MapToError<Size4KiB>> {
    // Reserve a guard page to prevent the stack from reading into data it
    // shouldn't
    let guard_page = reserve_stack_memory(size_in_pages + 1);

    let stack_start = guard_page + 1;
    let stack_end = stack_start + size_in_pages;

    for page in Page::range(stack_start, stack_end) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    Ok(StackBounds {
        start: stack_start.start_address(),
        end: stack_end.start_address(),
    })
}

/// Reserve the specified amount of virtual memory. Returns the start
/// page.
fn reserve_stack_memory(size_in_pages: u64) -> Page {
    static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x_5555_5555_0000);
    let start_addr = VirtAddr::new(
        STACK_ALLOC_NEXT.fetch_add(size_in_pages * Page::<Size4KiB>::SIZE, Ordering::Relaxed),
    );
    Page::from_start_address(start_addr).expect("`STACK_ALLOC_NEXT` not page aligned")
}
