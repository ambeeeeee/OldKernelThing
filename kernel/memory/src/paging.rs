use x86_64::registers::control::Cr3;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::{structures::paging::PageTable, VirtAddr};

/// Initialize a new `OffsetPageTable`
///
/// # Safety
///
/// The caller must guarantee that the complete physical memory is mapper to the
/// virtual memory at the passed `physical_memory_offset`. Additionally, this
/// function must be only called once.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_four_table = active_level_4_table(physical_memory_offset);

    OffsetPageTable::new(level_four_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 paging table.
///
/// # Safety
///
/// The caller must guarantee that the complete physical memory is
/// mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called
/// once to avoid aliasing `&mut` references (which is undefined
/// behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // Read directly from the paging control register
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // SAFETY: The caller was warned about aliasing `&mut` references
    &mut *page_table_ptr
}
