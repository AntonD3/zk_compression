//!
//! The circuit free implementation of compressing and uncompressing
//!

pub const ADDRESS_SIZE: u8 = 20;
pub const KEY_VALUE_SIZE: u8 = 32;

///
/// Storage transition rust representation.
///
pub struct StorageTransition {
    /// The account address.
    pub address: [u8; 20],
    /// The storage key.
    pub key: [u8; 32],
    /// The value.
    pub value: [u8; 32],
    /// The information for dynamic types encoding(optional).
    pub meta: Option<([u8; 32], [u8; 32])>,
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
}