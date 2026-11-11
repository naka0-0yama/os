pub mod entry;
pub mod handlers;

use core::arch::asm;
use core::mem::size_of;

use entry::IdtEntry;

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 32] = [const { IdtEntry::missing() }; 32];

unsafe extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
}

pub fn init() {
    let handlers = [
        isr0 as *const () as usize as u64,
        isr1 as *const ()as usize as u64,
        isr2 as *const ()as usize as u64,
        isr3 as *const ()as usize as u64,
        isr4 as *const ()as usize as u64,
        isr5 as *const ()as usize as u64,
        isr6 as *const ()as usize as u64,
        isr7 as *const ()as usize as u64,
        isr8 as *const ()as usize as u64,
        isr9 as *const ()as usize as u64,
        isr10 as *const () as usize as u64,
        isr11 as *const () as usize as u64,
        isr12 as *const () as usize as u64,
        isr13 as *const () as usize as u64,
        isr14 as *const () as usize as u64,
        isr15 as *const () as usize as u64,
        isr16 as *const () as usize as u64,
        isr17 as *const () as usize as u64,
        isr18 as *const () as usize as u64,
        isr19 as *const () as usize as u64,
        isr20 as *const () as usize as u64,
        isr21 as *const () as usize as u64,
        isr22 as *const () as usize as u64,
        isr23 as *const () as usize as u64,
        isr24 as *const () as usize as u64,
        isr25 as *const () as usize as u64,
        isr26 as *const () as usize as u64,
        isr27 as *const () as usize as u64,
        isr28 as *const () as usize as u64,
        isr29 as *const () as usize as u64,
        isr30 as *const () as usize as u64,
        isr31 as *const () as usize as u64,
    ];

    unsafe {
        let table = core::ptr::addr_of_mut!(IDT);
        let entries = table.cast::<IdtEntry>();
        
        for (index, &handler) in handlers.iter().enumerate() {
            entries.add(index).write(IdtEntry::new(handler));
        }

        let idtr = Idtr {
            limit: (size_of::<[IdtEntry; 32]>() - 1) as u16,
            base: table as u64,
        };

        asm!(
            "lidt [{}]",
            in(reg) &idtr,
            options(readonly, nostack, preserves_flags)
        );
    }
}
