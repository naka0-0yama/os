use crate::drivers::serial::COM1;

#[repr(C)]
pub struct ExceptionFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub vector: u64,
    pub error_code: u64,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
}

const EXCEPTION_NAMES: [&str; 32] = [
    "Divide Error",
    "Debug",
    "NMI",
    "Breakpoint",
    "Overflow",
    "BOUND Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
];

#[unsafe(no_mangle)]
pub extern "C" fn exception_dispatch(frame: *const ExceptionFrame) -> ! {
    let frame = unsafe { &*frame };

    if frame.vector == 14 {
        let cr2: u64;

        unsafe {
            core::arch::asm!(
                "mov {}, cr2",
                out(reg) cr2,
                options(nomem, nostack, preserves_flags)
            );
        }

        COM1.emergency_write_fmt(format_args!(
            "\r\n[PAGE FAULT]\r\n\
                     CR2: {:#018X}\r\n\
                     Error Code: {:#018X}\r\n\
                     RIP: {:#018X}\r\n",
            cr2, frame.error_code, frame.rip,
        ));
    } else {
        let name = EXCEPTION_NAMES
            .get(frame.vector as usize)
            .copied()
            .unwrap_or("Unknown Exception");

        COM1.emergency_write_fmt(format_args!(
            "\r\n[EXCEPTION]\r\n\
                     Vector: {}\r\n\
                     Name: {}\r\n\
                     Error Code: {:#018X}\r\n\
                     RIP: {:#018X}\r\n",
            frame.vector, name, frame.error_code, frame.rip,
        ));
    }

    loop {
        unsafe {
            core::arch::asm!("cli", "hlt", options(nomem, nostack));
        }
    }
}
