// https://git.shipyard.rs/jstrong/const-crc32/src/branch/master/src/lib.rs

/// used to generate up a [u32; 256] lookup table in `crc32`. this computes
/// the table on demand for a given "index" `i`
#[rustfmt::skip]
const fn table_fn(i: u32) -> u32 {
    let mut out = i;
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out
}
const fn get_table() -> [u32; 256] {
    let mut table: [u32; 256] = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = table_fn(i as u32);
        i += 1;
    }
    table
}
const TABLE: [u32; 256] = get_table();

pub fn crc32(buf: &[u8]) -> u32 {
    let seed = 0u32;
    let mut out = !seed;
    let mut i = 0usize;
    while i < buf.len() {
        out = (out >> 8) ^ TABLE[((out & 0xff) ^ (buf[i] as u32)) as usize];
        i += 1;
    }
    !out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let input = [0u8, 0u8, 0u8, 0u8];
        let result = crc32(&input);
        // https://crccalc.com/?crc=0x00,0x00,0x00,0x00&method=crc32&datatype=hex&outtype=0
        assert_eq!(result, 0x2144DF1C);

        // https://crccalc.com/?crc=hoge&method=crc32&datatype=ascii&outtype=0
        let input = "hoge".as_bytes();
        let result = crc32(&input);
        assert_eq!(result, 0x8B39E45A);
    }
}
