#![no_std]

use core::intrinsics::transmute;

use bootloader::BootInfo;
use frame_allocator_bootinfo::FrameAllocatorBootInfo;
use spin::Mutex;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{page_table::FrameError, OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};
pub mod allocator;
pub mod frame_allocator_bootinfo;
pub mod paging;
pub mod stack;

use lazy_static::lazy_static;

// Create global allocator and mapper
//
// Used in the rest of the kernel to make allocations
lazy_static! {
    pub static ref MAPPER: Mutex<Option<SendWrapper<OffsetPageTable<'static>>>> = Mutex::new(None);
    pub static ref FRAME_ALLOCATOR: Mutex<Option<SendWrapper<FrameAllocatorBootInfo>>> =
        Mutex::new(None);
}

pub struct SendWrapper<T>(T);

unsafe impl Send for SendWrapper<OffsetPageTable<'static>> {}
unsafe impl Send for SendWrapper<FrameAllocatorBootInfo> {}

impl<T> core::ops::Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> core::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn init_allocator(boot_info: &BootInfo) {
    let yolo: &'static BootInfo = unsafe { transmute(boot_info) };

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());

    let mapper: OffsetPageTable<'static> = unsafe { paging::init(phys_mem_offset) };

    let frame_allocator = unsafe { FrameAllocatorBootInfo::init(yolo) };

    // Set the global kernel mapper
    let mut mapper_static = MAPPER.lock();
    *mapper_static = Some(SendWrapper(mapper));

    // Set the global frame allocator
    let mut frame_allocator_static = FRAME_ALLOCATOR.lock();
    *frame_allocator_static = Some(SendWrapper(frame_allocator));
}

pub fn translate_address(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        // convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

pub fn with_mapper_and_allocator<F, T>(f: F) -> T
where
    F: FnOnce(&mut x86_64::structures::paging::OffsetPageTable, &mut FrameAllocatorBootInfo) -> T,
{
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut mapper_lock = MAPPER.lock();
        let mapper = mapper_lock.as_mut().unwrap();
        let mut frame_allocator_lock = FRAME_ALLOCATOR.lock();
        let frame_allocator = frame_allocator_lock.as_mut().unwrap();

        f(mapper, frame_allocator)
    })
}
