//!
//! The circuit free implementation of compressing and uncompressing
//!
mod utils;
#[cfg(test)]
mod tests;

pub const ADDRESS_SIZE: usize = 20;
pub const KEY_VALUE_SIZE: usize = 32;

///
/// Storage transition rust representation.
///
#[derive(Default, Debug, Clone, PartialEq)]
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

            let mut key = utils::compress_leading_zeroes(transition.key);

            if let Some(meta) = transition.meta {
                let preimage = utils::compress_leading_zeroes(meta.0);
                let offset = utils::compress_leading_zeroes(meta.1);
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

            let mut value = utils::compress_leading_zeroes(transition.value);
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
                let (preimage, offset) = utils::uncompress_leading_zeroes(&data[ptr+1..]);
                ptr += 1 + offset as usize;
                let (image_offset, offset) = utils::uncompress_leading_zeroes(&data[ptr..]);
                ptr += offset as usize;
                Value::KEY_VALUE(utils::slot_from_preimage_and_offset(preimage, image_offset))
            } else {
                let (value, offset) = utils::uncompress_leading_zeroes(&data[ptr..]);
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
        result.pop().expect("Always valid");
        result
    }
}
