//! Set membership zero-knowledge proof implementation for u64 type.
//! Keep in mind that this ZKP only proves set membership and does not prevent sending
//! the same value twice or sending he wrong value.
//!
//! TODO: Needs rigorous testing before actual use.

use super::merkle::{self, MerkleProof, MerkleTree};
use super::set_membership_circuit::SetMembershipCircuit;

use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::utils::byte_ops::convert_u8_to_u64;
use halo2_proofs::circuit::Value;
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier, VerifyingKey,
};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use thiserror::Error;
use tracing::debug;

/// Error type for Merkle Tree operations.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred during Merkle tree operations.
    #[error("Merkle tree error: {}", .0)]
    Merkle(#[from] merkle::Error),
    /// An invalid index had been used to access the set.
    #[error("Invalid set index: {}/{}", .0, .1)]
    InvalidIndex(usize, usize),
    /// Verification of the proof failed, indicating that the proof is invalid.
    #[error("Proof verification error: {}", .0)]
    Verification(#[from] halo2_proofs::plonk::Error),
    /// An empty set had been passed when creating new prover.
    #[error("Set cannot be empty")]
    EmptySet,
    /// Failed to serialize parameters, or verifying key.
    #[error("Parameters or verifying key serialization failed {}", .0)]
    Serialization(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Parameters used to generate and verify the proof.
/// Parameters are kept in their original form, but could be kept as bytes if ram will be a bigger
/// issue than verification speed.
/// This struct should be passed around as reference, since it is large and expensive to generate.
#[derive(Debug, Serialize, Deserialize)]
pub struct SetMembershipParams {
    /// Parameters used to generate and verify the proof.
    #[serde(with = "halo2_params")]
    inner: Params<EqAffine>,
}

/// Custom Serde serialization and deserialization for Halo2 parameters.
pub mod halo2_params {
    use serde::{Deserializer, Serializer};

    use super::*;

    /// Serializing Halo2 parameters to bytes and then passing it to Serde serializer.
    pub fn serialize<S>(
        inner: &Params<EqAffine>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut byte_buf = Vec::new();
        inner.write(&mut byte_buf).map_err(|e| {
            serde::ser::Error::custom(format!("Writing Halo2 params to bytes failed: {e}"))
        })?;
        serializer.serialize_bytes(&byte_buf)
    }

    /// Deserializing Halo2 parameters from serde dedserializer to bytes and reading it to actual params.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Params<EqAffine>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer).map_err(|e| {
            serde::de::Error::custom(format!(
                "Failed to deserialize into Halo2 params bytes from Serde obejct to bytes: {e}"
            ))
        })?;
        Params::<EqAffine>::read(&mut std::io::Cursor::new(bytes)).map_err(|e| {
            serde::de::Error::custom(format!(
                "Failed to deserialize into Halo2 params from bytes: {e}"
            ))
        })
    }
}

// TODO figure out if any of these functions will block.
impl SetMembershipParams {
    /// Creates a new instance of the parameters.
    /// The value of k is hardcoded here to be 10, since that's what works with the underlying Halo2 circuit.
    /// Theoretically this function should only be called once, the resulting struct stored
    /// and passed around as reference, because the params are both expensive to generate and large.
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize the parameters to a writer as bytes.
    pub fn write<W: std::io::Write>(&self, buf: &mut W) -> Result<()> {
        self.inner.write(buf)?;

        Ok(())
    }

    /// Deserialize the parameters from bytes from a reader.
    pub fn read<R: std::io::Read>(buf: &mut R) -> Result<Self> {
        let inner = Params::read(buf)?;
        Ok(Self { inner })
    }

    /// Get the inner parameters.
    pub fn get_inner(&self) -> &Params<EqAffine> {
        &self.inner
    }
}

impl Default for SetMembershipParams {
    /// Default implementation for params with k = 10.
    fn default() -> Self {
        // The value of k is specific for the circuit, so it is hardcoded here.
        let k = 10;
        let inner = Params::new(k);
        debug!("New Halo2 params created for set membership ZKP with k = {k}");

        Self { inner }
    }
}

/// All required info to prove that a given element is a member of the set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMembershipProof {
    /// Verification key used to verify the proof.
    vk: Vec<u8>,
    /// The actual proof that the element is a member of the set in bytes.
    proof: Vec<u8>,
}

impl SetMembershipProof {
    /// Proves that the element at the given index is a member of the set.
    /// Merkle tree and set are passed by reference to avoid large memory usage
    ///
    /// # Note
    ///
    /// This function is blocking, so use .spawn_blocking() ir it's equivalent,
    /// if you want to run it in an async context.
    ///
    /// # Arguments
    ///
    /// - `index` - The index of the element in the set.
    /// - `set` - The set of elements.
    /// - `merkle_tree` - The Merkle tree of the set.
    ///
    /// # Returns
    ///
    /// The proof that the element at the given index is a member of the set.
    ///
    /// # Errors
    ///
    /// An error will be returned if the index is invalid or if the proof generation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use crypto::set_membership_zkp::poseidon_hasher::{self, Digest};
    /// use crypto::set_membership_zkp::set_membership::SetMembershipProof;
    /// use crypto::set_membership_zkp::merkle::MerkleTree;
    /// use crypto::set_membership_zkp::set_membership::SetMembershipParams;
    ///
    /// let set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    /// let merkle_tree = MerkleTree::<u64, [u8; 32]>::new(
    ///     &set,
    ///     Box::new(|a, b| poseidon_hasher::hash([Digest(*a), Digest(*b)]).0),
    ///     Box::new(|x| poseidon_hasher::hash([x.into(), x.into()]).0),
    /// ).unwrap();
    /// let params = SetMembershipParams::new();
    /// let set_membership_proof = SetMembershipProof::new_blocking(5, &set, &merkle_tree, &params).unwrap();
    /// ```
    pub fn new_blocking(
        index: usize,
        set: &[u64],
        merkle_tree: &MerkleTree<u64, [u8; 32]>,
        params: &SetMembershipParams,
    ) -> Result<SetMembershipProof> {
        let MerkleProof { proof, path, .. } = merkle_tree.get_proof(index)?;

        let value = Value::known(
            set.get(index)
                .ok_or(Error::InvalidIndex(index, set.len()))?
                .to_owned()
                .into(),
        );
        let proof: Vec<Value<Fp>> = proof
            .iter()
            .map(|x| Value::known(Fp::from_raw(convert_u8_to_u64(x.to_owned()))))
            .collect();
        let path: Vec<Value<Fp>> = path
            .iter()
            .map(|x| Value::known(Fp::from(bool::from(x.to_owned()))))
            .collect();
        let root = Fp::from_raw(convert_u8_to_u64(merkle_tree.get_root()));

        let circuit = SetMembershipCircuit::new(value, proof, path);

        let vk = keygen_vk(params.get_inner(), &circuit)?;
        let pk = keygen_pk(params.get_inner(), vk.clone(), &circuit)?;
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        create_proof(
            params.get_inner(),
            &pk,
            &[circuit],
            &[&[&[root], &[Fp::zero()]]],
            OsRng,
            &mut transcript,
        )?;

        let proof = transcript.finalize();
        let vk = vk.to_bytes();
        debug!("Set membership ZKP proof and VK created for item {index}");

        Ok(SetMembershipProof { vk, proof })
    }

    /// Verifies the proof that the unknown element is a member of the set.
    ///
    /// # Note
    ///
    /// This function is blocking, so use .spawn_blocking() ir it's equivalent,
    /// if you want to run it in an async context.
    ///
    /// # Arguments
    ///
    /// - `merkle_root` - The Merkle root of the set.
    ///
    /// # Returns
    ///
    /// An empty result if the proof is valid.
    ///
    /// # Errors
    ///
    /// An error will be returned if the proof verification fails.
    ///
    /// # Example
    ///
    /// ```
    /// use crypto::set_membership_zkp::poseidon_hasher::{self, Digest};
    /// use crypto::set_membership_zkp::set_membership::SetMembershipProof;
    /// use crypto::set_membership_zkp::merkle::MerkleTree;
    /// use crypto::set_membership_zkp::set_membership::SetMembershipParams;
    ///
    /// let set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    /// let merkle_tree = MerkleTree::<u64, [u8; 32]>::new(
    ///     &set,
    ///     Box::new(|a, b| poseidon_hasher::hash([Digest(*a), Digest(*b)]).0),
    ///     Box::new(|x| poseidon_hasher::hash([x.into(), x.into()]).0),
    /// ).unwrap();
    /// let params = SetMembershipParams::new();
    /// let set_membership_proof = SetMembershipProof::new_blocking(5, &set, &merkle_tree, &params).unwrap();
    /// set_membership_proof.verify_blocking(merkle_tree.get_root(), &params).unwrap();
    /// ```
    pub fn verify_blocking(
        &self,
        merkle_root: [u8; 32],
        params: &SetMembershipParams,
    ) -> Result<()> {
        let vk = VerifyingKey::<EqAffine>::from_bytes::<SetMembershipCircuit>(
            &self.vk,
            params.get_inner(),
        )
        .unwrap();
        let mut transcript =
            Blake2bRead::<_, _, Challenge255<_>>::init(std::io::Cursor::new(&self.proof));
        let root = Fp::from_raw(convert_u8_to_u64(merkle_root));
        let res = Ok(verify_proof(
            params.get_inner(),
            &vk,
            SingleVerifier::new(params.get_inner()),
            &[&[&[root], &[Fp::zero()]]],
            &mut transcript,
        )?);
        debug!("Set membership ZKP proof verified");

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::super::poseidon_hasher::{self, Digest};

    #[wasm_bindgen_test]
    #[test]
    fn test_prove_and_verify() {
        let params = SetMembershipParams::new();
        let set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let merkle_tree = MerkleTree::<u64, [u8; 32]>::new(
            &set,
            Box::new(|a, b| poseidon_hasher::hash([Digest(*a), Digest(*b)]).0),
            Box::new(|x| poseidon_hasher::hash([x.into(), x.into()]).0),
        )
        .unwrap();
        let merkle_root = merkle_tree.get_root();

        let set_membership_proof =
            SetMembershipProof::new_blocking(5, &set, &merkle_tree, &params).unwrap();
        let mut test_buf = Vec::new();
        params.write(&mut test_buf).unwrap();

        set_membership_proof
            .verify_blocking(merkle_root, &params)
            .unwrap();
    }
}
