#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod arch;
mod console;
mod font;
use crate::arch::x86_64::acpi;

use crate::arch::x86_64::{boot, serial};

#[unsafe(no_mangle)]
pub extern "C" fn rust_main(magic: u32, multiboot_info_addr: usize) -> ! {
    serial::init();
    boot::init(magic, multiboot_info_addr);

    println!("Hello, world!");
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
fn panic(_info: &PanicInfo) -> ! {
    halt();
}
