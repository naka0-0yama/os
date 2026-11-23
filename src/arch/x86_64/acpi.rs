use crate::println;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AcpiHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GenericAddressStructure {
    pub address_space: u8,
    pub bit_width: u8,
    pub bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FADT {
    pub header: AcpiHeader,

    pub firmware_ctrl: u32,
    pub dsdt: u32,

    // ACPI 1.0 compatibility
    pub reserved: u8,

    pub preferred_power_management_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,

    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,

    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,

    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,

    pub worst_c2_latency: u16,
    pub worst_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,

    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,

    // ACPI 2.0+
    pub boot_architecture_flags: u16,

    pub reserved2: u8,
    pub flags: u32,

    pub reset_reg: GenericAddressStructure,

    pub reset_value: u8,
    pub reserved3: [u8; 3],

    // ACPI 2.0+
    pub x_firmware_control: u64,
    pub x_dsdt: u64,

    pub x_pm1a_event_block: GenericAddressStructure,
    pub x_pm1b_event_block: GenericAddressStructure,
    pub x_pm1a_control_block: GenericAddressStructure,
    pub x_pm1b_control_block: GenericAddressStructure,
    pub x_pm2_control_block: GenericAddressStructure,
    pub x_pm_timer_block: GenericAddressStructure,
    pub x_gpe0_block: GenericAddressStructure,
    pub x_gpe1_block: GenericAddressStructure,
}

pub fn parse_xsdt(xsdt_addr: u64) {
    let header_ptr = xsdt_addr as *const AcpiHeader;
    let header = unsafe { core::ptr::read_unaligned(header_ptr) };
    if header.signature != *b"XSDT" {
        println!("Invalid XSDT signature");
        return;
    }
    let length = header.length as usize;
    let header_size = core::mem::size_of::<AcpiHeader>();

    let entry_count = (length - header_size) / 8;

    let pointers_ptr =  (xsdt_addr as usize + header_size) as * const u64;
    println!("--- Parsing XSDT (Entries: {}) ---", entry_count);
    for i in 0..entry_count {
        let table_paddr = unsafe { core::ptr::read_unaligned(pointers_ptr.add(i)) };
        
        // 【重要】4GB以上（0x100000000以上）または 0 のアドレスは現在のページング外なのでスキップする
        if table_paddr == 0 || table_paddr >= 0x1_0000_0000 {
            println!("  [{}] Skipping out-of-range address: 0x{:x}", i, table_paddr);
            continue;
        }

        let table_header_ptr = table_paddr as *const AcpiHeader;
        let table_header = unsafe { core::ptr::read_unaligned(table_header_ptr) };
        let sig = table_header.signature;

        let sig_str = core::str::from_utf8(&sig).unwrap_or("????");
        println!("  [{}] Table Signature: {}, Address: 0x{:x}", i, sig_str, table_paddr);

        if sig == *b"FACP" {
            println!("  [{}] Found FADT at address: 0x{:x}", i, table_paddr);
            let facp_ptr = table_paddr as *const FADT;
            let facp = unsafe { core::ptr::read_unaligned(facp_ptr)};
            
        }
    }
}