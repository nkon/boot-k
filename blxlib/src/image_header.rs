use crate::crc32::crc32;
use core::ptr;

pub const HEADER_LENGTH: u16 = 256;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageHeader {
    pub header_magic: u32,  // 4
    pub header_length: u16, // +2 = 6
    pub hv_major: u8,       // +1 = 7
    pub hv_minor: u8,       // +1 = 8

    pub iv_major: u8,  // +1 = 9
    pub iv_minor: u8,  // +1 = 10
    pub iv_patch: u16, // +2 = 12
    pub iv_build: u32, // +4 = 16

    pub image_length: u32,    // +4 = 20
    pub signature: [u8; 128], // +128 = 148

    pub payload_crc: u32,   // +4 = 152
    pub padding: [u8; 100], // +100 = 252

    pub crc32: u32, // +4 = 256
}

pub fn load_from_addr(addr: u32) -> ImageHeader {
    unsafe { ptr::read_volatile(addr as *const ImageHeader) }
}

pub fn load_from_buf(buf: &[u8]) -> ImageHeader {
    unsafe { *(buf.as_ptr() as *const ImageHeader) }
}

pub fn as_bytes_with_len<T: ?Sized>(t: &T, len: usize) -> &[u8] {
    unsafe {
        let p = t as *const _ as *const u8;
        &*ptr::slice_from_raw_parts(p, len)
    }
}

impl Default for ImageHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageHeader {
    pub fn new() -> Self {
        ImageHeader {
            header_magic: 0xb00410ad,
            header_length: HEADER_LENGTH,
            hv_major: 0,
            hv_minor: 0,
            iv_major: 0,
            iv_minor: 0,
            iv_patch: 0,
            iv_build: 0,
            image_length: 0xe_0000,
            signature: [0u8; 128],
            payload_crc: 0,
            padding: [0u8; 100],
            crc32: 0,
        }
    }
    pub fn is_correct_magic(&self) -> bool {
        self.header_magic == 0xb00410ad
    }
    pub fn set_crc32(&mut self) {
        let buf = as_bytes_with_len(self, HEADER_LENGTH as usize - 4);
        let crc32 = crc32(buf);
        self.crc32 = crc32;
    }
    pub fn is_correct_crc(&self) -> bool {
        let buf = as_bytes_with_len(self, HEADER_LENGTH as usize - 4);
        let crc32 = crc32(buf);
        self.crc32 == crc32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_correct_magic() {
        let mut ih = ImageHeader::new();
        let result = ih.is_correct_magic();
        assert_eq!(result, true);

        ih.header_magic = 0;
        let result = ih.is_correct_magic();
        assert_eq!(result, false);
    }

    #[test]
    fn test_set_crc32() {
        let mut ih = ImageHeader::new();
        ih.header_magic = 0;
        ih.header_length = 0;
        ih.hv_major = 0;
        ih.hv_minor = 0;
        ih.iv_major = 0;
        ih.iv_minor = 0;
        ih.iv_patch = 0;
        ih.iv_build = 0;
        ih.image_length = 0;
        ih.signature = [0u8; 128];
        ih.payload_crc = 0;
        ih.padding = [0u8; 100];

        ih.set_crc32();
        // https://crccalc.com/?crc=0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00&method=crc32&datatype=hex&outtype=0
        assert_eq!(ih.crc32, 0xA66359F1);
    }
}
