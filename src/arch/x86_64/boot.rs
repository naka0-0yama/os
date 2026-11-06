use crate::console;
use crate::println;
const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36d76289;
const TAG_TYPE_END: u32 = 0;
const TAG_TYPE_FRAMEBUFFER: u32 = 8;
const TAG_TYPE_MEMMMAP: u32 = 17;

#[repr(C)]
struct MultibootInfo {
    total_size: u32,
    reserved: u32,
}

#[repr(C)]
struct Tag {
    tag_type: u32,
    size: u32,
}

#[repr(C)]
struct FramebufferTag {
    tag_type: u32,
    size: u32,
    address: u64,
    pitch: u32,
    width: u32,
    height: u32,
    bpp: u8,
    framebuffer_type: u8,
    reserved: u16,
    red_position: u8,
    red_mask_size: u8,
    green_position: u8,
    green_mask_size: u8,
    blue_position: u8,
    blue_mask_size: u8,
}
#[repr(C)]
struct MemmapTag {
    tag_type: u32,
    size: u32,
    descriptor_size: u32,
    descriptor_version: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EfiMemoryDesc {
    pub ty: u32,
    pub pad: u32,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_count: u64,
    pub attribute: u64,
}

/// ブート情報の検証と初期化（Multiboot2の詳細をここに隠蔽）
pub fn init(magic: u32, multiboot_info_addr: usize) {
    if magic != MULTIBOOT2_BOOTLOADER_MAGIC {
        crate::halt();
    }

    let info = multiboot_info_addr as *const MultibootInfo;
    let total_size = unsafe { (*info).total_size as usize };
    let mut offset = 8usize;

    let mut fb_tag_ptr: Option<*const FramebufferTag> = None;
    let mut memmap_tag_ptr: Option<*const MemmapTag> = None;

    while offset < total_size {
        let tag = unsafe { &*((multiboot_info_addr + offset) as *const Tag) };
        if tag.tag_type == TAG_TYPE_END {
            break;
        }
        if tag.tag_type == TAG_TYPE_FRAMEBUFFER {
            fb_tag_ptr = Some((multiboot_info_addr + offset) as *const FramebufferTag);
        } else if tag.tag_type == TAG_TYPE_MEMMMAP {
            memmap_tag_ptr = Some((multiboot_info_addr + offset) as *const MemmapTag);
        }
        offset += (tag.size as usize + 7) & !7;
    }
    if let Some(fb_ptr) = fb_tag_ptr {
        let fb = unsafe { &*fb_ptr };
        unsafe {
            console::init(
                fb.address as *mut u32,
                fb.width as usize,
                fb.height as usize,
            );
        }
        println!(
            "Framebuffer: {:x}, {}x{}, {} bpp",
            fb.address, fb.width, fb.height, fb.bpp
        );
    }
    if let Some(mmap_ptr) = memmap_tag_ptr {
        let efi_mmap_tag = unsafe { &*mmap_ptr };
        let desc_size = efi_mmap_tag.descriptor_size as usize;
        let header_size = core::mem::size_of::<MemmapTag>();
        let desc_count = (efi_mmap_tag.size as usize - header_size) / desc_size;

        let mut desc_ptr = ((mmap_ptr as usize) + header_size) as *const EfiMemoryDesc;
        println!("Memory Map:");
        for _ in 0..desc_count {
            let desc = unsafe { &*desc_ptr };
            let is_ram = desc.ty == 7;
            let type_str = if is_ram {
                "RAM  (Conventional)"
            } else {
                "Reserved/Other"
            };
            let length = desc.page_count * 4096;
            println!(
                "Base: 0x{:016x}, Len: 0x{:016x} [{}]",
                desc.phys_start, length, type_str
            );
            desc_ptr = ((desc_ptr as usize) + desc_size) as *const EfiMemoryDesc;
        }
    }
}
