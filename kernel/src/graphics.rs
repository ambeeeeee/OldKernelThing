use core::mem::MaybeUninit;

use alloc::boxed::Box;
use bootloader::{boot_info::FrameBuffer, BootInfo};
use driver_vga::VGADriver;
use vga_efi::VgaEfi;

pub static mut VGA_BACKEND: MaybeUninit<*const (dyn VGADriver + Send + Sync)> =
    MaybeUninit::uninit();

pub fn setup_graphics(boot_info: &'static BootInfo) {
    let frame_buffer: &FrameBuffer = boot_info.framebuffer.as_ref().unwrap();

    let info = frame_buffer.info();

    let frame_buffer_data = frame_buffer.buffer();

    let frame_buffer_pointer = frame_buffer_data as *const [u8] as *mut [u8];

    let renderer = VgaEfi::new(frame_buffer_pointer, info).unwrap();

    let renderer = Box::into_raw(Box::new(renderer) as _) as *const _;

    let mut swap_src = MaybeUninit::new(renderer);

    core::mem::swap(&mut swap_src, unsafe { &mut VGA_BACKEND });
}

pub fn get_graphics() -> *const (dyn VGADriver + Send + Sync) {
    unsafe { VGA_BACKEND.assume_init() }
}

pub fn graphics_ref<'a>() -> &'a (dyn VGADriver + Send + Sync) {
    unsafe { &*VGA_BACKEND.assume_init() }
}
