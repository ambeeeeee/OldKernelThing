#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod serial;

pub mod acpi;
pub mod console;
pub mod gdt;
pub mod graphics;
pub mod idt;

#[no_mangle]
fn fminf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[no_mangle]
fn fmaxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}
