use crate::println;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
const SLP_TYP_SHIFT: u16 = 10;
const SLP_TYP_MASK: u16 = 0x7;
const SLP_EN: u16 = 1 << 13;

// GenericAddressStructure の address_space 値
const ADDR_SPACE_SYSTEM_MEMORY: u8 = 0;
const ADDR_SPACE_SYSTEM_IO: u8 = 1;

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
#[allow(clippy::upper_case_acronyms)]
#[repr(C, packed)]
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

#[derive(Clone, Copy, Debug)]
pub struct S5Values {
    pub slp_typ_a: u8,
    pub slp_typ_b: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum ControlReg {
    IoPort(u16),
    Mmio(u64),
}

#[derive(Clone, Copy, Debug)]
pub struct AcpiShutdownInfo {
    pub pm1a: Option<ControlReg>,
    pub pm1b: Option<ControlReg>,
    pub s5: S5Values,
}

struct SyncUnsafeCell<T>(UnsafeCell<T>);

// シングルコア初期化またはロック/Atomicフラグによる順序制御を前提とするラッパー
unsafe impl<T: Copy> Sync for SyncUnsafeCell<T> {}

static SHUTDOWN_READY: AtomicBool = AtomicBool::new(false);
static SHUTDOWN_INFO: SyncUnsafeCell<Option<AcpiShutdownInfo>> =
    SyncUnsafeCell(UnsafeCell::new(None));

/// XSDTパース時に抽出したシャットダウン情報を登録する
pub fn register_shutdown_info(info: AcpiShutdownInfo) {
    unsafe {
        *SHUTDOWN_INFO.0.get() = Some(info);
    }
    SHUTDOWN_READY.store(true, Ordering::Release);
}

/// 登録済みの ACPI 情報を使ってシャットダウンを実行する
pub fn shutdown() -> ! {
    if SHUTDOWN_READY.load(Ordering::Acquire) {
        let info = unsafe { *SHUTDOWN_INFO.0.get() };
        if let Some(cfg) = info {
            println!(
                "Shutting down via ACPI: PM1a={:?}, PM1b={:?}, S5={:?}",
                cfg.pm1a, cfg.pm1b, cfg.s5
            );
            unsafe {
                core::arch::asm!("cli", options(nomem, nostack));
                if let Some(reg_a) = cfg.pm1a {
                    write_pm1_control(reg_a, cfg.s5.slp_typ_a);
                }
                if let Some(reg_b) = cfg.pm1b {
                    write_pm1_control(reg_b, cfg.s5.slp_typ_b);
                }
            }
        }
    } else {
        println!("ACPI shutdown info not available; halting CPU.");
    }

    halt_forever()
}

pub fn halt_forever() -> ! {
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

pub fn find_s5_value(dsdt_bytes: &[u8]) -> Option<S5Values> {
    let mut i = 0;
    while i + 4 <= dsdt_bytes.len() {
        if &dsdt_bytes[i..i + 4] == b"_S5_"
            && let Some(vals) = parse_s5_after_name(&dsdt_bytes[i + 4..])
        {
            return Some(vals);
        }
        i += 1;
    }
    None
}

fn parse_s5_after_name(mut rest: &[u8]) -> Option<S5Values> {
    let mut pkg_idx = None;
    for (idx, &b) in rest.iter().take(8).enumerate() {
        if b == 0x12 {
            pkg_idx = Some(idx);
            break;
        }
    }
    rest = &rest[pkg_idx?..];

    if rest.first().copied()? != 0x12 {
        return None;
    }
    rest = &rest[1..];

    let pkg_len_consumed = skip_pkg_length(rest)?;
    rest = &rest[pkg_len_consumed..];

    let num_elements = *rest.first()? as usize;
    rest = &rest[1..];

    if num_elements == 0 {
        return None;
    }

    let (slp_typ_a, next_rest) = parse_aml_integer(rest)?;
    rest = next_rest;

    let slp_typ_b = if num_elements >= 2 {
        parse_aml_integer(rest).map(|(v, _)| v).unwrap_or(slp_typ_a)
    } else {
        slp_typ_a
    };

    Some(S5Values {
        slp_typ_a,
        slp_typ_b,
    })
}

fn skip_pkg_length(buf: &[u8]) -> Option<usize> {
    let lead = *buf.first()?;
    let extra_bytes = ((lead >> 6) & 0x03) as usize;
    let total = 1 + extra_bytes;
    if buf.len() < total {
        return None;
    }
    Some(total)
}

fn parse_aml_integer(buf: &[u8]) -> Option<(u8, &[u8])> {
    let op = *buf.first()?;
    match op {
        0x00 => Some((0, &buf[1..])),

        0x01 => Some((1, &buf[1..])),

        0xFF => Some((0xFF, &buf[1..])),

        0x0A => {
            let val = *buf.get(1)?;
            Some((val, &buf[2..]))
        }
        0x0B => {
            let lo = *buf.get(1)?;
            let _hi = *buf.get(2)?;
            Some((lo, &buf[3..]))
        }
        _ => None,
    }
}

fn get_dsdt_paddr(facp: &FADT) -> u64 {
    let x_dsdt = facp.x_dsdt;
    if x_dsdt != 0 {
        x_dsdt
    } else {
        facp.dsdt as u64
    }
}

unsafe fn read_dsdt_bytes<'a>(dsdt_paddr: u64) -> Option<&'a [u8]> {
    if dsdt_paddr == 0 || dsdt_paddr >= 0x1_0000_0000 {
        return None;
    }

    let hdr_ptr = dsdt_paddr as *const AcpiHeader;
    let hdr = unsafe { core::ptr::read_unaligned(hdr_ptr) };
    if &hdr.signature != b"DSDT" {
        return None;
    }

    let len = hdr.length as usize;
    let hdr_size = core::mem::size_of::<AcpiHeader>();
    if len < hdr_size {
        return None;
    }

    // DSDT全体（ヘッダ + AMLバイト列）をスライス化
    unsafe { Some(core::slice::from_raw_parts(dsdt_paddr as *const u8, len)) }
}

/// FADT から PM1a 制御レジスタのアクセス先を取得する
fn get_pm1a_control_reg(facp: &FADT) -> Option<ControlReg> {
    let x_blk = facp.x_pm1a_control_block;
    let x_addr = { x_blk.address };
    let x_space = { x_blk.address_space };

    if x_addr != 0 {
        return match x_space {
            ADDR_SPACE_SYSTEM_IO => u16::try_from(x_addr).ok().map(ControlReg::IoPort),
            ADDR_SPACE_SYSTEM_MEMORY => Some(ControlReg::Mmio(x_addr)),
            _ => None,
        };
    }

    let legacy_port = { facp.pm1a_control_block };
    if legacy_port != 0 {
        u16::try_from(legacy_port).ok().map(ControlReg::IoPort)
    } else {
        None
    }
}

/// FADT から PM1b 制御レジスタのアクセス先を取得する（存在しない場合は None）
fn get_pm1b_control_reg(facp: &FADT) -> Option<ControlReg> {
    let x_blk = facp.x_pm1b_control_block;
    let x_addr = { x_blk.address };
    let x_space = { x_blk.address_space };

    if x_addr != 0 {
        return match x_space {
            ADDR_SPACE_SYSTEM_IO => u16::try_from(x_addr).ok().map(ControlReg::IoPort),
            ADDR_SPACE_SYSTEM_MEMORY => Some(ControlReg::Mmio(x_addr)),
            _ => None,
        };
    }

    let legacy_port = { facp.pm1b_control_block };
    if legacy_port != 0 {
        u16::try_from(legacy_port).ok().map(ControlReg::IoPort)
    } else {
        None
    }
}

/// x86 16bit I/O ポート読み込み（outw 時に既存ビットを温存したい場合用）
fn inw(port: u16) -> u16 {
    let val: u16;
    unsafe {
        core::arch::asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") val,
            options(nomem, nostack, preserves_flags)
        );
    }
    val
}

/// x86 16bit I/O ポート書き込み
fn outw(port: u16, val: u16) {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") val,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// 制御レジスタに 16bit 値を書き込む（MMIO を使う場合はページング有効範囲/HHDM に注意）
fn write_pm1_control(reg: ControlReg, slp_typ: u8) {
    let typ_bits = ((slp_typ as u16) & SLP_TYP_MASK) << SLP_TYP_SHIFT;

    match reg {
        ControlReg::IoPort(port) => {
            // 必要に応じて既存値を読み出し、SLP_TYP (10..12) と SLP_EN (13) を更新
            let cur = inw(port);
            let next = (cur & !(SLP_TYP_MASK << SLP_TYP_SHIFT)) | typ_bits | SLP_EN;
            outw(port, next);
        }
        ControlReg::Mmio(paddr) => {
            // 現在のページングがアイデンティティマッピング前提の例
            // HHDM を使う場合は paddr + hhdm_offset に変換してください
            let ptr = paddr as *mut u16;
            let cur = unsafe { core::ptr::read_volatile(ptr) };
            let next = (cur & !(SLP_TYP_MASK << SLP_TYP_SHIFT)) | typ_bits | SLP_EN;
            unsafe { core::ptr::write_volatile(ptr, next) };
        }
    }
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

    let pointers_ptr = (xsdt_addr as usize + header_size) as *const u64;
    println!("--- Parsing XSDT (Entries: {}) ---", entry_count);
    for i in 0..entry_count {
        let table_paddr = unsafe { core::ptr::read_unaligned(pointers_ptr.add(i)) };

        // 【重要】4GB以上（0x100000000以上）または 0 のアドレスは現在のページング外なのでスキップする
        if table_paddr == 0 || table_paddr >= 0x1_0000_0000 {
            println!(
                "  [{}] Skipping out-of-range address: 0x{:x}",
                i, table_paddr
            );
            continue;
        }

        let table_header_ptr = table_paddr as *const AcpiHeader;
        let table_header = unsafe { core::ptr::read_unaligned(table_header_ptr) };
        let sig = table_header.signature;

        let sig_str = core::str::from_utf8(&sig).unwrap_or("????");
        println!(
            "  [{}] Table Signature: {}, Address: 0x{:x}",
            i, sig_str, table_paddr
        );

        if sig == *b"FACP" {
            println!("  [{}] Found FADT at address: 0x{:x}", i, table_paddr);
            let facp_ptr = table_paddr as *const FADT;
            let facp = unsafe { core::ptr::read_unaligned(facp_ptr) };
            let dsdt_paddr = get_dsdt_paddr(&facp);
            println!("  [{}] DSDT Physical Address: 0x{:x}", i, dsdt_paddr);

            if let Some(dsdt_bytes) = unsafe { read_dsdt_bytes(dsdt_paddr) } {
                if let Some(s5) = find_s5_value(dsdt_bytes) {
                    println!(
                        "  [{}] Found _S5_ values: SLP_TYPa = {}, SLP_TYPb = {}",
                        i, s5.slp_typ_a, s5.slp_typ_b
                    );
                    let val_a = (((s5.slp_typ_a as u16) & 0x7) << 10) | (1 << 13);
                    println!(
                        "  [{}] Found _S5_ values: SLP_TYPa = {}, SLP_TYPb = {} (PM1a write=0x{:04x})",
                        i, s5.slp_typ_a, s5.slp_typ_b, val_a
                    );
                    register_shutdown_info(AcpiShutdownInfo {
                        pm1a: get_pm1a_control_reg(&facp),
                        pm1b: get_pm1b_control_reg(&facp),
                        s5,
                    });
                } else {
                    println!("  [{}] _S5_ not found in DSDT", i);
                }
            } else {
                println!("  [{}] Failed to read DSDT bytes", i);
            }
        }
    }
}
