//! Module containing a more user friendly version of the the Poseidon hash function from the halo2_gadgets crate
//! And the Digest type for more convenient interoperability with other types.
//!
//! Note that reusing the Poseidon hasher from the Halo2 crate turned to be pretty hacky, due to halo2's types being
//! hard to convert to other types.

use crate::utils::byte_ops::convert_u8_to_u64;
use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
use halo2_proofs::pasta::Fp;

// TODO After trying multiple different approaches (array, uin256 crates, etc), couldn't figure a way to
// avoid having to create a new type for the hash value, since all existing types would be very unwieldy to use
// so will have to try to figure this out some more in the future, but commiting this struct for now.
/// Struct to abstract the hash value to be more interoperable with other types.
/// The length of the hash is fixed to 32 bytes, because Poseidon hash operates on fields.
/// The inner value can be accessed directly since it's public.
#[derive(Default, Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct Digest(pub [u8; 32]);

impl TryFrom<&[u8]> for Digest {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            return Err("Invalid input length");
        }
        let mut internal = Self::default();
        internal.0.copy_from_slice(value);
        Ok(internal)
    }
}

impl From<Fp> for Digest {
    fn from(fp: Fp) -> Self {
        Digest(fp.into())
    }
}

impl From<Digest> for Fp {
    fn from(val: Digest) -> Fp {
        Fp::from_raw(convert_u8_to_u64(val.0))
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for Digest {
    fn from(value: [u8; 32]) -> Self {
        Digest(value)
    }
}

impl From<u64> for Digest {
    fn from(value: u64) -> Self {
        let mut buf = [0u8; 32];
        buf[..8].copy_from_slice(&value.to_le_bytes());
        Digest(buf)
    }
}

impl From<&u64> for Digest {
    fn from(value: &u64) -> Self {
        let mut buf = [0u8; 32];
        buf[..8].copy_from_slice(&value.to_le_bytes());
        Digest(buf)
    }
}

/// Hashes the input using the Poseidon hash function provided by Halo2 crate.
/// The main intention for this function is to hash the input data for the
/// Merkle tree leaf nodes.
///
/// # Note
///
/// Because Poseidon hash needs two inputs, using a copy of the single input of this
/// function as padding for the empty second input of the hash algorithm.
///
/// # Arguments
///
/// * `input` - The input data to be hashed as bytes. The length of the input is fixed to 32 bytes,
///             because Poseidon hash operates on fields instead of regular numbers.
///
/// # Returns
///
/// * `digest` - The hash of the input data abstracted in a Digest type.
///
/// # Example
///
/// ```
/// use digital_voting::set_membership::poseidon_hasher;
/// use digital_voting::set_membership::poseidon_hasher::Digest;
///
/// let input = [Digest([1u8; 32]), Digest([2u8; 32])];
///
/// let digest = poseidon_hasher::hash(input);
/// ```
pub fn hash(input: [Digest; 2]) -> Digest {
    let input = [
        Fp::from_raw(convert_u8_to_u64(input[0].0)),
        Fp::from_raw(convert_u8_to_u64(input[1].0)),
    ];
    let digest =
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([input[0], input[1]]);
    digest.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO, need to find out test values and make sure there aren't any bugs in the poseidon hash or the implementation around it.
    #[ignore]
    #[test]
    fn test_poseidon_hasher_test_values() {
        // TODO currently these values are wrong, need to find the correct ones.
        let input = [Fp::from(1234567890), Fp::from(1234567890)];
        let expected_result: [u8; 32] = [
            0x15, 0x09, 0x82, 0x90, 0x1a, 0xd0, 0x84, 0x45, 0xad, 0x2d, 0x8f, 0x63, 0xed, 0x38,
            0xa0, 0x98, 0xb5, 0xd1, 0x88, 0x4b, 0x52, 0x6e, 0x0d, 0xf7, 0x73, 0x95, 0xdf, 0xa0,
            0x3d, 0xc8, 0x19, 0x93,
        ];
        let digest: [u8; 32] = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash(input)
            .into();
        assert_eq!(expected_result, digest);
    }
}
