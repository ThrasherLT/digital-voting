//! Set membership zero-knowledge proof implementation for u64 type.

// TODO proof needs to be serializable.
// TODO needs rigorous testing before actual use.

use super::merkle::{self, MerkleProof, MerkleTree};
use super::set_membership_circuit::SetMembershipCircuit;

use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
use halo2_proofs::poly::ipa::multiopen::ProverIPA;
use halo2_proofs::poly::VerificationStrategy;
use rand::rngs::OsRng;

use crate::utils::byte_ops::convert_u8_to_u64;
use halo2_proofs::circuit::Value;
use halo2_proofs::halo2curves::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, VerifyingKey};
use halo2_proofs::poly::ipa::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use thiserror::Error;

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
}
type Result<T> = std::result::Result<T, Error>;

/// All required info to prove that a given element is a member of the set.
#[derive(Debug, Clone)]
pub struct SetMembershipProof {
    /// Verification key used to verify the proof.
    vk: VerifyingKey<EqAffine>,
    /// Parameters used to generate and verify the proof.
    params: ParamsIPA<EqAffine>,
    /// The actual proof that the element is a member of the set in bytes.
    proof: Vec<u8>,
}

impl SetMembershipProof {
    /// Proves that the element at the given index is a member of the set.
    /// Merkle tree and set are passed by reference to avoid large memory usage
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
    /// use digital_voting::set_membership_zkp::poseidon_hasher::{self, Digest};
    /// use digital_voting::set_membership_zkp::set_membership::SetMembershipProof;
    /// use digital_voting::set_membership_zkp::merkle::MerkleTree;
    ///
    /// let set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    /// let merkle_tree = MerkleTree::<u64, [u8; 32]>::new(
    ///     &set,
    ///     Box::new(|a, b| poseidon_hasher::hash([Digest(*a), Digest(*b)]).0),
    ///     Box::new(|x| poseidon_hasher::hash([x.into(), x.into()]).0),
    /// ).unwrap();
    /// let set_membership_proof = SetMembershipProof::new(5, &set, &merkle_tree).unwrap();
    /// ```
    pub fn new(
        index: usize,
        set: &[u64],
        merkle_tree: &MerkleTree<u64, [u8; 32]>,
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

        let params = ParamsIPA::<EqAffine>::new(10);
        let vk = keygen_vk(&params, &circuit)?;
        let pk = keygen_pk(&params, vk.clone(), &circuit)?;
        let vvk = vk.pinned();
        println!("{:#?}", vvk);
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);

        create_proof::<IPACommitmentScheme<_>, ProverIPA<_>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[&[root], &[Fp::zero()]]],
            OsRng,
            &mut transcript,
        )?;

        let proof = transcript.finalize();

        Ok(SetMembershipProof { vk, params, proof })
    }

    /// Verifies the proof that the unknown element is a member of the set.
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
    /// use digital_voting::set_membership_zkp::poseidon_hasher::{self, Digest};
    /// use digital_voting::set_membership_zkp::set_membership::SetMembershipProof;
    /// use digital_voting::set_membership_zkp::merkle::MerkleTree;
    ///
    /// let set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    /// let merkle_tree = MerkleTree::<u64, [u8; 32]>::new(
    ///     &set,
    ///     Box::new(|a, b| poseidon_hasher::hash([Digest(*a), Digest(*b)]).0),
    ///     Box::new(|x| poseidon_hasher::hash([x.into(), x.into()]).0),
    /// ).unwrap();
    /// let set_membership_proof = SetMembershipProof::new(5, &set, &merkle_tree).unwrap();
    /// set_membership_proof.verify(merkle_tree.get_root()).unwrap();
    /// ```
    pub fn verify(&self, merkle_root: [u8; 32]) -> Result<()> {
        let mut transcript =
            Blake2bRead::<_, _, Challenge255<_>>::init(std::io::Cursor::new(&self.proof));
        let root = Fp::from_raw(convert_u8_to_u64(merkle_root));

        Ok(verify_proof(
            &self.params,
            &self.vk,
            SingleStrategy::new(&self.params),
            &[&[&[root], &[Fp::zero()]]],
            &mut transcript,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::poseidon_hasher::{self, Digest};

    #[test]
    fn test_prove_and_verify() {
        let set = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let merkle_tree = MerkleTree::<u64, [u8; 32]>::new(
            &set,
            Box::new(|a, b| poseidon_hasher::hash([Digest(*a), Digest(*b)]).0),
            Box::new(|x| poseidon_hasher::hash([x.into(), x.into()]).0),
        )
        .unwrap();
        let merkle_root = merkle_tree.get_root();

        let set_membership_proof = SetMembershipProof::new(5, &set, &merkle_tree).unwrap();
        set_membership_proof.verify(merkle_root).unwrap();
    }
}
