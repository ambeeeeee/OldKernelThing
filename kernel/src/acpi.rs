use core::{
    mem,
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use alloc::sync::Arc;
use bootloader::BootInfo;
use kernel_memory::with_mapper_and_allocator;
use x86_64::{
    structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub static mut ACPI_TABLES: Option<AcpiTables<Handler>> = None;

pub fn get_acpi_tables() -> &'static AcpiTables<Handler> {
    unsafe { ACPI_TABLES.as_ref().unwrap() }
}

#[derive(Clone)]
pub struct Handler {
    phsyical_offset: Arc<VirtAddr>,
}

impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        with_mapper_and_allocator(|mapper, allocator| {
            let start = PhysAddr::new(physical_address as u64);
            let start_page = PhysFrame::containing_address(start);

            let phys_frames = {
                let end = start + mem::size_of::<T>();

                let end_page = PhysFrame::containing_address(end + size);

                PhysFrame::range_inclusive(start_page, end_page + 1)
            };

            let page_start = what_the_fuck(size as u64 / Size4KiB::SIZE + 1);
            let pages =
                { Page::range_inclusive(page_start, page_start + (size as u64 / Size4KiB::SIZE)) };

            for (page, frame) in pages.zip(phys_frames) {
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

                unsafe {
                    mapper
                        .map_to(page, frame, flags, allocator)
                        .unwrap()
                        .flush()
                }
            }

            let ret_ptr = page_start.start_address() + (start - start.align_down(Size4KiB::SIZE));

            PhysicalMapping::new(
                start_page.start_address().as_u64() as usize,
                NonNull::new(ret_ptr.as_mut_ptr()).unwrap(),
                size,
                69,
                self.clone(),
            )
        })
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        with_mapper_and_allocator(|mapper, _| {
            let start = Page::<Size4KiB>::containing_address(VirtAddr::from_ptr(
                region.virtual_start().as_ptr(),
            ));

            let page_range = {
                let end_page =
                    Page::containing_address(start.start_address() + region.region_length());

                Page::range_inclusive(start, end_page)
            };

            for page in page_range {
                mapper.unmap(page).expect("failed to unmap page").1.flush();
            }
        });
    }
}

pub fn init(boot_info: &'static BootInfo) -> AcpiTables<Handler> {
    let base = VirtAddr::new(*boot_info.physical_memory_offset.as_ref().unwrap());

    let handler = Handler {
        phsyical_offset: Arc::new(base),
    };

    unsafe { AcpiTables::search_for_rsdp_bios(handler).unwrap() }
}

fn what_the_fuck(size_in_pages: u64) -> Page {
    static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x_6969_5555_0000);
    let start_addr = VirtAddr::new(
        STACK_ALLOC_NEXT.fetch_add(size_in_pages * Page::<Size4KiB>::SIZE, Ordering::Relaxed),
    );
    Page::from_start_address(start_addr).expect("`STACK_ALLOC_NEXT` not page aligned")
}
