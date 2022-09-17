//!
//! The circuit free implementation of compressing and uncompressing
//!

///
/// Storage transition implemention
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
