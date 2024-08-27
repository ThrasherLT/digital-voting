//! This module contains the implementation of the SetMembershipCircuit struct, which is a
//! circuit that proves that a given leaf is a member of a Merkle tree.
//! The circuit uses the Poseidon hash function to hash the leaf and the Merkle proof elements.
//! Poseidon hash function is used instead of sha256, because it's a lot faster in ZKP circuits
//! and because halo2's support for sha256 is very limited.

use halo2_gadgets::poseidon::primitives::P128Pow5T3;
use halo2_proofs::{circuit::*, pasta::Fp, plonk::*, poly::Rotation};

use super::poseidon_chip::{PoseidonChip, PoseidonConfig};

/// halo2 circuit that proves that a given leaf is a member of a set.
#[derive(Debug, Clone, Default)]
pub struct SetMembershipCircuit {
    /// The leaf value (not hashed yet) that is being proven to be a member of the set.
    value: Value<Fp>,
    /// The Merkle proof elements that are used to prove the membership of the leaf.
    merkle_proof: Vec<Value<Fp>>,
    /// The directions of the Merkle proof elements.
    /// If the direction is 0, the proof element is on the right side of the hash.
    direction: Vec<Value<Fp>>,
}

/// Configuration for the SetMembershipCircuit.
#[derive(Debug, Clone)]
pub struct SetMembershipConfig {
    /// The advice columns for the SetMembershipCircuit.
    advices: [Column<Advice>; 3],
    /// Selector for enforcing boolean values.
    bool_selector: Selector,
    /// The swap selector for switching digest and proof sides depending on direction of hashing.
    swap_selector: Selector,
    /// The instance column which will contain the root of the merkle tree.
    instance: Column<Instance>,
    /// The configuration for the Poseidon hash function.
    poseidon_config: PoseidonConfig<3, 2, 2>,
}

impl SetMembershipCircuit {
    /// Create a new SetMembershipCircuit with the given leaf value, Merkle proof elements and directions.
    ///
    /// # Arguments
    ///
    /// - `value` - The leaf value (not hashed yet) that is being proven to be a member of the set.
    /// - `merkle_proof` - The Merkle proof elements that are used to prove the membership of the leaf.
    /// - `direction` - The directions of the Merkle proof elements.
    ///
    /// # Returns
    ///
    /// A new SetMembershipCircuit instance.
    ///
    /// # Example
    ///
    /// ```
    /// use halo2_proofs::circuit::Value;
    /// use crypto::set_membership_zkp::set_membership_circuit::SetMembershipCircuit;
    ///
    /// let value = Value::known(halo2_proofs::pasta::Fp::from(6u64));
    /// let merkle_proof = vec![
    ///     Value::known(halo2_proofs::pasta::Fp::from(1u64)),
    ///     Value::known(halo2_proofs::pasta::Fp::from(2u64)),
    /// ];
    /// let direction = vec![
    ///   Value::known(halo2_proofs::pasta::Fp::from(0u64)),
    ///  Value::known(halo2_proofs::pasta::Fp::from(1u64)),
    /// ];
    /// let circuit = SetMembershipCircuit::new(value, merkle_proof, direction);
    /// ```
    pub fn new(value: Value<Fp>, merkle_proof: Vec<Value<Fp>>, direction: Vec<Value<Fp>>) -> Self {
        Self {
            value,
            merkle_proof,
            direction,
        }
    }

    /// Function containing most of the proving logic for set membership.
    fn prove(
        &self,
        config: SetMembershipConfig,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let mut digest = layouter.assign_region(
            || "initialize",
            |mut region| {
                region.assign_advice(|| "assign value", config.advices[0], 0, || self.value)
            },
        )?;
        // Initial hash of the leaf preimage value. Since Poseidon hasher takes two inputs, we duplicate the value.
        let poseidon_hash_chip =
            PoseidonChip::<P128Pow5T3, 3, 2, 2>::new(config.poseidon_config.clone());
        digest = poseidon_hash_chip.hash(&mut layouter, &[digest.clone(), digest])?;

        for i in 0..self.merkle_proof.len() {
            let (lhs, rhs) = layouter.assign_region(
                || "prove",
                |mut region| {
                    digest.copy_advice(|| "assign value", &mut region, config.advices[0], 0)?;
                    region.assign_advice(
                        || "assign proof",
                        config.advices[1],
                        0,
                        || self.merkle_proof[i],
                    )?;
                    region.assign_advice(
                        || "assign direction",
                        config.advices[2],
                        0,
                        || self.direction[i],
                    )?;

                    config.bool_selector.enable(&mut region, 0)?;
                    config.swap_selector.enable(&mut region, 0)?;
                    let digest_owned_value = digest.value().map(|x| x.to_owned());
                    let (mut lhs, mut rhs) = (digest_owned_value, self.merkle_proof[i]);
                    self.direction[i].map(|direction| {
                        if direction == Fp::one() {
                            (lhs, rhs) = (self.merkle_proof[i], digest_owned_value);
                        }
                    });

                    let lhs =
                        region.assign_advice(|| "assign lhs", config.advices[0], 1, || lhs)?;
                    let rhs =
                        region.assign_advice(|| "assign rhs", config.advices[1], 1, || rhs)?;

                    Ok((lhs, rhs))
                },
            )?;

            let poseidon_hash_chip =
                PoseidonChip::<P128Pow5T3, 3, 2, 2>::new(config.poseidon_config.clone());
            digest = poseidon_hash_chip.hash(&mut layouter, &[lhs, rhs])?;
        }
        layouter.constrain_instance(digest.cell(), config.instance, 0)?;

        Ok(())
    }
}

impl Circuit<Fp> for SetMembershipCircuit {
    type Config = SetMembershipConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<Fp>) -> Self::Config {
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        let instance = meta.instance_column();
        let bool_selector = meta.selector();
        let swap_selector = meta.selector();

        for advice_column in advices {
            meta.enable_equality(advice_column);
        }
        meta.enable_equality(instance);

        meta.create_gate("bool", |meta| {
            let bool_selector = meta.query_selector(bool_selector);
            let direction = meta.query_advice(advices[2], Rotation::cur());
            vec![
                bool_selector * (direction.clone() * (direction - Expression::Constant(Fp::one()))),
            ]
        });

        meta.create_gate("swap", |meta| {
            let swap_selector = meta.query_selector(swap_selector);

            let our_element = meta.query_advice(advices[0], Rotation::cur());
            let proof_element = meta.query_advice(advices[1], Rotation::cur());
            let direction = meta.query_advice(advices[2], Rotation::cur());

            let lhs = meta.query_advice(advices[0], Rotation::next());
            let rhs = meta.query_advice(advices[1], Rotation::next());

            vec![
                swap_selector
                    * (direction
                        * Expression::Constant(Fp::from(2))
                        * (proof_element.clone() - our_element.clone())
                        - (lhs - our_element)
                        - (proof_element - rhs)),
            ]
        });

        SetMembershipConfig {
            advices,
            bool_selector,
            swap_selector,
            instance,
            poseidon_config: PoseidonChip::<P128Pow5T3, 3, 2, 2>::configure(meta),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl halo2_proofs::circuit::Layouter<Fp>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.prove(config, layouter)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::poseidon_hasher;
    use crate::utils::byte_ops::convert_u8_to_u64;
    use halo2_proofs::{circuit::Value, dev::MockProver, pasta::Fp};

    // Function to calculate the root of a Merkle tree proof manually.
    // Using this to avoid having to set up a full blown Merkle tree.
    fn calculate_root_manually(leaf: &u64, elements: &Vec<u64>, indices: &Vec<u64>) -> [u8; 32] {
        let mut digest = poseidon_hasher::hash([leaf.to_owned().into(), leaf.to_owned().into()]);
        for i in 0..elements.len() {
            if indices[i] == 0 {
                digest = poseidon_hasher::hash([digest.0.into(), elements[i].into()]);
            } else {
                digest = poseidon_hasher::hash([elements[i].into(), digest.0.into()]);
            }
        }
        return digest.0;
    }

    #[test]
    fn test_circuit_legit() {
        let leaf = 6u64;
        let elements = vec![1u64, 2u64, 3u64, 4u64, 5u64];
        let indices = vec![0u64, 1u64, 0u64, 0u64, 1u64];

        let digest = calculate_root_manually(&leaf, &elements, &indices);

        let elements_fp: Vec<Value<Fp>> = elements
            .iter()
            .map(|x| Value::known(Fp::from(x.to_owned())))
            .collect();
        let indices_fp: Vec<Value<Fp>> = indices
            .iter()
            .map(|x| Value::known(Fp::from(x.to_owned())))
            .collect();
        let leaf_fp = Value::known(Fp::from(leaf));
        let circuit = SetMembershipCircuit::new(leaf_fp, elements_fp, indices_fp);
        let root_fp = Fp::from_raw(convert_u8_to_u64(digest));

        let prover = MockProver::run(10, &circuit, vec![vec![root_fp], vec![Fp::zero()]]).unwrap();
        // Using assert_satisfied() instead of verify() because the former pretty prints verification failures.
        prover.assert_satisfied();
    }

    #[test]
    fn test_circuit_falsified() {
        let leaf = 6u64;
        let elements = vec![1u64, 2u64, 3u64, 4u64, 5u64];
        let indices = vec![0u64, 1u64, 0u64, 0u64, 1u64];

        let digest = calculate_root_manually(&leaf, &elements, &indices);

        let elements_fp: Vec<Value<Fp>> = elements
            .iter()
            .map(|x| Value::known(Fp::from(x.to_owned())))
            .collect();
        let indices_fp: Vec<Value<Fp>> = indices
            .iter()
            .map(|x| Value::known(Fp::from(x.to_owned())))
            .collect();
        let leaf_fp = Value::known(Fp::from(leaf));
        let circuit = SetMembershipCircuit::new(leaf_fp, elements_fp, indices_fp);
        let root_fp = Fp::from_raw(convert_u8_to_u64(digest));

        let prover = MockProver::run(
            10,
            &circuit,
            vec![vec![root_fp + Fp::one()], vec![Fp::zero()]],
        )
        .unwrap();
        assert!(prover.verify().is_err())
    }
}
