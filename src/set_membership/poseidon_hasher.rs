//! Module containing the Poseidon hasher and it's trait implementations required for it to work with the Merkle tree
//! from the merkle_light crate. Also providing a standalone Poseidon hash function for convenience.
//!
//! Note that reusing the Poseidon hasher from the Halo2 crate turned to be pretty hacky, so will probably have to
//! be reworked in the future, but will remain for now.
//!
//! Besides that merkle_light also proved to be quite messy and doing a custom merkle tree might be nice in the future.

use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use merkle_light::hash::Algorithm;
use std::hash::Hasher;

/// Struct to abstract the hash value to be more interoperable with other types.
/// The length of the hash is fixed to 32 bytes, because Poseidon hash operates on fields.
/// The inner calue can be accessed directly since it's public.
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

impl InternalHash for Digest {}

// TODO there's probably a better place for this:
fn convert_u8_to_u64(input: [u8; 32]) -> [u64; 4] {
    let mut output = [0u64; 4];

    for (i, item) in output.iter_mut().enumerate() {
        let start = i * 8;
        let end = start + 8;
        *item = u64::from_le_bytes(input[start..end].try_into().unwrap());
    }

    output
}

/// Struct containing the hasher state to be used to plug the Poseidon hasher provided
/// by the Halo2 crate into the Merkle tree provided by the merkle_light crate.
#[derive(Debug, Default, PartialEq)]
pub struct PoseidonHasher {
    /// Counter for how many entries have been written to the hasher's input.
    /// Used to prevent writing more than 2 entries.
    /// In the future might be used to handle leaf nodes, when count is 1.
    input_entry_count: usize,
    /// The actual data to be hashed abstracted with the Digest struct for simpler
    /// interoperability with other types. An array is used to avoid heap allocation.
    input: [Digest; 2],
}

impl PoseidonHasher {
    // TODO it'd probablt be nice to have a more ergonomic input, since rust can be annoying about arrays.
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
    /// use digital_voting::set_membership::poseidon_hasher::PoseidonHasher;
    ///
    /// let input = [1u8; 32];
    ///
    /// let digest = PoseidonHasher::hash(input);
    /// ```
    pub fn hash(input: [u8; 32]) -> Digest {
        let input = Fp::from_raw(convert_u8_to_u64(input));
        let digest =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([input, input]);
        digest.into()
    }
}

/// This trait is needed by the `Algorithm` trait so this hasher can be used with merkle_light crate.
impl Hasher for PoseidonHasher {
    fn finish(&self) -> u64 {
        // merkle_light does not require this method.
        unimplemented!()
    }

    fn write(&mut self, bytes: &[u8]) {
        if self.input_entry_count >= 2 {
            // TODO trace
            return;
        }
        if bytes.len() > 32 {
            // TODO trace
            return;
        }
        // expect is fine here, since it's handled by the if statement above.
        let input: Digest = bytes
            .try_into()
            .expect("Invalid Poseidon hasher input length");
        self.input[self.input_entry_count] = input;
        self.input_entry_count += 1;
    }
}

trait InternalHash: AsRef<[u8]> + Clone + Copy + From<[u8; 32]> {}

// TODO this turned out to be a lot more hacky than I initially wanted:
/// This is the trait that merkle_light uses to plug in external hasher implementations.
impl<T: InternalHash> Algorithm<T> for PoseidonHasher {
    /// Returns the Poseidon hash value for the expected two data inputs.
    /// Called by merkle_light crate.
    ///
    /// # Returns
    ///
    /// * `digest` - The hash of the two inputs abstracted by Digest type.
    ///
    /// # Panics
    ///
    /// Panics if the input count is not 1 or 2, since Poseidon hash requires two inputs.
    fn hash(&mut self) -> T {
        let input: [Fp; 2] = match self.input_entry_count {
            // Padding Poseidon hash input with the same element since it needs two elements:
            1 => [self.input[0].into(), self.input[0].into()],
            2 => [self.input[0].into(), self.input[1].into()],
            // Should never happen:
            _ => panic!("Invalid Poseidon hasher input count"),
        };
        // TODO Poseidon configuration should be passed as a parameter or at least be a constant:
        let digest = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(input);
        let digest_bytes: [u8; 32] = digest.into();
        T::from(digest_bytes)
    }

    /// Reset Hasher state.
    #[inline]
    fn reset(&mut self) {
        *self = Self::default();
    }

    /// No need for extra operations to be done on an already hashed leaf.
    ///
    /// # Note
    ///
    /// merkle_light's documentation is really lacking and this method is actually called
    /// on leaves that are already hashed by the method in the `Hasher` trait.
    ///
    /// # Arguments
    ///
    /// * `leaf` - A leaf of the Merkle tree. This method is used for actions on the already hashed leaf,
    ///            but nothing is needed for this implementation.
    ///
    /// # Returns
    ///
    /// * `leaf` - The same leaf that was passed as an argument.
    #[inline]
    fn leaf(&mut self, leaf: T) -> T {
        leaf
    }

    /// Returns the hash value for two neighboring nodes in the Merkle tree.
    ///
    /// # Arguments
    ///
    /// * `left` - The left node (hash) of the Merkle tree.
    /// * `right` - The right node (hash) of the Merkle tree.
    ///
    /// # Returns
    ///
    /// * `hash` - The hash of the two neighboring nodes.
    #[inline]
    fn node(&mut self, left: T, right: T) -> T {
        self.write(left.as_ref());
        self.write(right.as_ref());
        self.hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use merkle_light::merkle::MerkleTree;

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

    // Checking if merkle tree handles the data (and the underlying bytes, endiness, etc) as expected.
    #[test]
    fn test_merkle_tree() {
        let elements = vec![1u64, 2u64, 3u64];
        let hash_iter = elements.iter().map(|x| {
            PoseidonHasher::hash({
                let mut array = [0u8; 32];
                array[..8].copy_from_slice(&x.to_le_bytes());
                array
            })
        });
        let tree = MerkleTree::<Digest, PoseidonHasher>::from_iter(hash_iter);
        let root = tree.root();

        let digest1 = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([Fp::from(1u64), Fp::from(1u64)]);
        let digest2 = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([Fp::from(2u64), Fp::from(2u64)]);
        let digest3 = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([Fp::from(3u64), Fp::from(3u64)]);
        let digest12 = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([digest1, digest2]);
        let digest33 = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([digest3, digest3]);
        let digest123: Digest = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([digest12, digest33])
            .into();

        assert_eq!(root, digest123);
    }
}
