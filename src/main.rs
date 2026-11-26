#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod arch;
mod console;
mod drivers;
mod font;
use crate::arch::x86_64::acpi;

use crate::arch::x86_64::boot;
use crate::arch::x86_64::idt;
use crate::drivers::serial;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main(magic: u32, multiboot_info_addr: usize) -> ! {
    serial::COM1.init();
    serialprintln!("[BOOT] COM1 initialized");
    println!("[BOOT] booting...");
    idt::init();
    println!("[BOOT] IDT initialized");
    boot::init(magic, multiboot_info_addr);
    println!("Hello, world!");
    println!("[BOOT] initialization complete!");

    acpi::shutdown();
}

pub fn halt() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::COM1.emergency_write_fmt(format_args!("\r\n[PANIC]\r\n{}\r\n", info));

    halt();
}
