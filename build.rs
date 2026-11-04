use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/arch/x86_64/boot.asm");
    println!("cargo:rerun-if-changed=src/arch/x86_64/linker.ld");

    let out_dir = env::var("OUT_DIR").unwrap();
    let boot_o = format!("{}/boot.o", out_dir);
    let libboot_a = format!("{}/libboot.a", out_dir);

    let nasm_status = Command::new("nasm")
        .args(["-f", "elf64", "src/arch/x86_64/boot.asm", "-o", &boot_o])
        .status()
        .expect("Failed to execute nasm. Is it installed?");

    if !nasm_status.success() {
        panic!("nasm compilation failed");
    }

    let ar_status = Command::new("ar")
        .args(["rcs", &libboot_a, &boot_o])
        .status()
        .expect("Failed to execute ar.");

    if !ar_status.success() {
        panic!("ar archiving failed");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=boot");
}
