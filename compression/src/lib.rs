//!
//! The circuit free implementation of compressing and uncompressing
//!
use sha3::Digest;

pub const ADDRESS_SIZE: usize = 20;
pub const KEY_VALUE_SIZE: usize = 32;

///
/// Storage transition rust representation.
///
#[derive(Default, Debug)]
pub struct StorageTransition {
    /// The account address.
    pub address: [u8; ADDRESS_SIZE],
    /// The storage key.
    pub key: [u8; KEY_VALUE_SIZE],
    /// The value.
    pub value: [u8; KEY_VALUE_SIZE],
    /// The information for dynamic types encoding(optional).
    pub meta: Option<([u8; KEY_VALUE_SIZE], [u8; KEY_VALUE_SIZE])>,
}

impl StorageTransition {
    ///
    /// Converts storage transitions array into bytes.
    ///
    pub fn into_bytes(transitions: Vec<Self>) -> Vec<u8> {
        let mut result = Vec::with_capacity(transitions.len() * (2*KEY_VALUE_SIZE + ADDRESS_SIZE));

        for transition in transitions {
            result.extend(transition.address);
            result.extend(transition.key);
            result.extend(transition.value);
        }

        result
    }

    ///
    /// Compress storage transitions
    ///
    pub fn compress(transitions: Vec<Self>) -> Vec<u8> {
        let mut result = Vec::new();

        for transition in transitions {
            result.push(1);
            result.extend(transition.address);

            let mut key = compress_leading_zeroes(transition.key);

            if let Some(meta) = transition.meta {
                let preimage = compress_leading_zeroes(meta.0);
                let offset = compress_leading_zeroes(meta.1);
                if preimage.len() + offset.len() + 1 < key.len() {
                    key = vec![2];
                    key.extend(preimage);
                    key.extend(offset);
                }
            }

            if key.len() >= KEY_VALUE_SIZE + 1 {
                key = vec![0];
                key.extend(transition.key);
            }

            result.extend(key);

            let mut value = compress_leading_zeroes(transition.value);
            if value.len() >= KEY_VALUE_SIZE + 1 {
                value = vec![0];
                value.extend(transition.value);
            }

            result.extend(value);

        }

        result
    }

    ///
    /// Uncompress storage transitions
    ///
    pub fn uncompress(data: Vec<u8>) -> Vec<Self> {
        let mut result = Vec::new();

        #[derive(Copy, Clone)]
        enum Field {
            ADDRESS,
            KEY,
            VALUE,
        }

        enum Value {
            #[allow(non_camel_case_types)]
            KEY_VALUE([u8; KEY_VALUE_SIZE]),
            ADDRESS([u8; ADDRESS_SIZE]),
        }

        let mut ptr = 0;
        let mut expected_field = Field::ADDRESS;
        result.push( Self::default());

        while ptr < data.len() {
            let value = if data[ptr] == 0 {
                let mut value = [0; KEY_VALUE_SIZE];
                for index in ptr+1..=ptr+KEY_VALUE_SIZE {
                    value[index - ptr - 1] = data[index];
                }
                ptr += KEY_VALUE_SIZE + 1;
                Value::KEY_VALUE(value)
            } else if data[ptr] == 1 {
                let mut value = [0; ADDRESS_SIZE];
                for index in ptr+1..=ptr+ADDRESS_SIZE {
                    value[index - ptr - 1] = data[index];
                }
                ptr += ADDRESS_SIZE + 1;
                Value::ADDRESS(value)
            } else if data[ptr] == 2 {
                let (preimage, offset) = uncompress_leading_zeroes(&data[ptr+1..]);
                ptr += 1 + offset as usize;
                let (image_offset, offset) = uncompress_leading_zeroes(&data[ptr..]);
                ptr += offset as usize;
                Value::KEY_VALUE(slot_from_preimage_and_offset(preimage, image_offset))
            } else {
                let (value, offset) = uncompress_leading_zeroes(&data[ptr..]);
                ptr += offset as usize;
                Value::KEY_VALUE(value)
            };

            match (expected_field, value) {
                (Field::ADDRESS, Value::ADDRESS(address)) => {
                    result.last_mut().expect("Always valid").address = address;
                    expected_field = Field::KEY;
                },
                (Field::KEY, Value::KEY_VALUE(value)) => {
                    result.last_mut().expect("Always valid").key = value;
                    expected_field = Field::VALUE;
                },
                (Field::VALUE, Value::KEY_VALUE(value)) => {
                    result.last_mut().expect("Always valid").value = value;
                    expected_field = Field::ADDRESS;
                    result.push(Self::default());
                },
                _ => panic!("Invalid compressed data")
            }
        }

        result
    }
}

///
/// Compress the value using first zeroes.
///
fn compress_leading_zeroes(value: [u8; KEY_VALUE_SIZE]) -> Vec<u8> {
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

fn uncompress_leading_zeroes(slice: &[u8]) -> ([u8; KEY_VALUE_SIZE], u8) {
    let zero_bytes = slice[0] - 10;

    assert!(zero_bytes > 10 && zero_bytes as usize <= KEY_VALUE_SIZE);

    let mut result = [0u8; KEY_VALUE_SIZE];
    for index in zero_bytes as usize..KEY_VALUE_SIZE {
        result[index] = slice[index-zero_bytes as usize + 1];
    }

    (result, 1 + KEY_VALUE_SIZE as u8 - zero_bytes)
}

fn slot_from_preimage_and_offset(preimage: [u8; KEY_VALUE_SIZE], offset: [u8; KEY_VALUE_SIZE]) -> [u8; KEY_VALUE_SIZE] {
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