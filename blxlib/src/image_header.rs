use core::ptr;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ImageHeader {
    pub header_magic: u32,  // 4
    pub header_length: u16, // +2 = 6
    pub hv_major: u8,       // +1 = 7
    pub hv_minor: u8,       // +1 = 8

    pub iv_major: u8,     // +1 = 9
    pub iv_minor: u8,     // +1 = 10
    pub iv_revision: u16, // +2 = 12
    pub iv_build: u32,    // +4 = 16

    pub image_length: u32,    // +4 = 20
    pub signature: [u8; 128], // +128 = 148

    pub padding: [u8; 104], // +104 = 252
    pub crc32: u32,         // +4 = 256
}

pub fn load_from_addr(addr: u32) -> ImageHeader {
    unsafe { ptr::read_volatile(addr as *const ImageHeader) }
}
