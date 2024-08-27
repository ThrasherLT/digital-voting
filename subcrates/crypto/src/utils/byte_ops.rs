//! This is a file for utility code regarding byte operations.

/// Convert a [u8; 32] array to a [u32; 8] array.
pub fn _convert_u8_to_u32(input: [u8; 32]) -> [u32; 8] {
    let mut output = [0u32; 8];

    for (i, item) in output.iter_mut().enumerate() {
        let start = i * 4;
        let end = start + 4;
        *item = u32::from_le_bytes(input[start..end].try_into().unwrap());
    }

    output
}

/// Convert a [u8; 32] array to a [u64; 4] array.
pub fn convert_u8_to_u64(input: [u8; 32]) -> [u64; 4] {
    let mut output = [0u64; 4];

    for (i, item) in output.iter_mut().enumerate() {
        let start = i * 8;
        let end = start + 8;
        *item = u64::from_le_bytes(input[start..end].try_into().unwrap());
    }

    output
}
