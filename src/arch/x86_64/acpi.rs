use crate::println;

#[repr(C, packed)]
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
            // FADT固有の解析（PM1aポートの取得など）を行う
        }
    }
}