#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use core::{fmt::Write, panic::PanicInfo};

use acpi::AcpiTables;
use bootloader::{entry_point, BootInfo};
use kernel::{
    acpi::{get_acpi_tables, ACPI_TABLES},
    console::setup_console,
    gdt,
    graphics::setup_graphics,
    idt, serial_println,
};
use kernel_memory::{allocator::init_heap, init_allocator, with_mapper_and_allocator};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Panicked at {}", info);
    loop {}
}

entry_point!(kmain);

fn kmain(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("Setting up GDT/TSS");
    gdt::init();
    serial_println!("[COMPLETE]");

    serial_println!("Setting up IDT");
    idt::init();
    serial_println!("[COMPLETE]");

    serial_println!("Set up paging");
    init_allocator(&boot_info);
    serial_println!("[COMPLETE]");

    serial_println!("Setup heap");
    with_mapper_and_allocator(|mapper, frame_allocator| {
        init_heap(mapper, frame_allocator).expect("Heap init Failed")
    });
    serial_println!("[COMPLETE]");

    serial_println!("Setup ACPI Tables");
    let tables = kernel::acpi::init(boot_info);

    unsafe { ACPI_TABLES = Some(tables) };
    serial_println!("[COMPLETE]");

    serial_println!("Setup graphics drivers.");
    setup_graphics(boot_info);
    serial_println!("[COMPLETE]");

    serial_println!("Get VGA console.");
    let mut console = setup_console();
    serial_println!("[COMPLETE]");

    writeln!(&mut console, "Graphics Initialized").unwrap();

    loop {}
}
