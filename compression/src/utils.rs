//!
//! Compression utils functions
//!

use super::KEY_VALUE_SIZE;

use sha3::Digest;

///
/// Compress the value using first zeroes.
///
pub fn compress_leading_zeroes(value: [u8; KEY_VALUE_SIZE]) -> Vec<u8> {
    let mut ptr = 0;
    let mut result = Vec::new();
    while ptr < KEY_VALUE_SIZE && value[ptr] == 0 {
        ptr += 1;
    }
    result.push(ptr as u8 + 10);
    for index in ptr..KEY_VALUE_SIZE {
        result.push(value[index]);
    }
    result
}

///
/// Uncompress the value with first zeroes.
///
pub fn uncompress_leading_zeroes(slice: &[u8]) -> ([u8; KEY_VALUE_SIZE], u8) {
    let zero_bytes = slice[0] - 10;

    assert!(zero_bytes > 10 && zero_bytes as usize <= KEY_VALUE_SIZE);

    let mut result = [0u8; KEY_VALUE_SIZE];
    for index in zero_bytes as usize..KEY_VALUE_SIZE {
        result[index] = slice[index-zero_bytes as usize + 1];
    }

    (result, 1 + KEY_VALUE_SIZE as u8 - zero_bytes)
}

pub fn slot_from_preimage_and_offset(preimage: [u8; KEY_VALUE_SIZE], offset: [u8; KEY_VALUE_SIZE]) -> [u8; KEY_VALUE_SIZE] {
    let image = sha3::Keccak256::digest(
        preimage.as_slice()
    ).as_slice().to_vec();
    assert_eq!(image.len(), KEY_VALUE_SIZE);

    let mut add = 0u16;
    let mut result = [0u8; KEY_VALUE_SIZE];
    let mut ptr = KEY_VALUE_SIZE - 1;
    loop {
        add += image[ptr] as u16;
        add += offset[ptr] as u16;
        result[ptr] = (add % 256) as u8;
        add /= 256;
        if ptr == 0 {
            break;
        } else {
            ptr -= 1;
        }
    }
    assert_eq!(add, 0);
    result
}