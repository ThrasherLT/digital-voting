//! Poseidon hash function chip implementation, because the one found in halo2_gadgets can't
//! be used in a circuit without a lot of boilerplate code.

use halo2_proofs::{circuit::*, pasta::Fp, plonk::*};

use halo2_gadgets::poseidon::{primitives::*, Hash, Pow5Chip, Pow5Config};
use std::convert::TryInto;
use std::marker::PhantomData;

/// The Poseidon hash function chip.
#[derive(Clone, Debug)]
pub struct PoseidonChip<S, const WIDTH: usize, const RATE: usize, const L: usize>
where
    S: Spec<Fp, WIDTH, RATE>,
{
    /// The configuration for the Poseidon hash function.
    config: PoseidonConfig<WIDTH, RATE, L>,
    /// The specification for the Poseidon hash function.
    _spec: PhantomData<S>,
}

/// Configuration for the Poseidon hash function.
#[derive(Debug, Clone)]
pub struct PoseidonConfig<const WIDTH: usize, const RATE: usize, const L: usize> {
    /// The input columns for the Poseidon hash function.
    inputs: [Column<Advice>; L],
    /// The expected output column for the Poseidon hash function.
    _expected: Column<Instance>,
    /// The configuration for the Poseidon hash function.
    poseidon_config: Pow5Config<Fp, WIDTH, RATE>,
}

impl<S, const WIDTH: usize, const RATE: usize, const L: usize> PoseidonChip<S, WIDTH, RATE, L>
where
    S: Spec<Fp, WIDTH, RATE>,
{
    /// Create a new PoseidonChip with the given configuration.
    ///
    /// # Arguments
    ///
    /// - `config` - The configuration for the Poseidon hash function.
    pub fn new(config: PoseidonConfig<WIDTH, RATE, L>) -> Self {
        Self {
            config,
            _spec: PhantomData,
        }
    }

    /// Configure the Poseidon hash function with the given constraint system.
    ///
    /// # Arguments
    ///
    /// - `meta` - The constraint system to configure the Poseidon hash function with.
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> PoseidonConfig<WIDTH, RATE, L> {
        let state = (0..WIDTH).map(|_| meta.advice_column()).collect::<Vec<_>>();
        let expected = meta.instance_column();
        meta.enable_equality(expected);
        let partial_sbox = meta.advice_column();

        let rc_a = (0..WIDTH).map(|_| meta.fixed_column()).collect::<Vec<_>>();
        let rc_b = (0..WIDTH).map(|_| meta.fixed_column()).collect::<Vec<_>>();

        meta.enable_constant(rc_b[0]);

        for item in state.iter().take(WIDTH) {
            meta.enable_equality(*item);
        }

        PoseidonConfig {
            inputs: state[..RATE].try_into().unwrap(),
            _expected: expected,
            poseidon_config: Pow5Chip::configure::<S>(
                meta,
                state.try_into().unwrap(),
                partial_sbox,
                rc_a.try_into().unwrap(),
                rc_b.try_into().unwrap(),
            ),
        }
    }

    /// Hash the given message words with the Poseidon hash function.
    ///
    /// # Arguments
    ///
    /// - `layouter` - The layouter to hash the message words with.
    /// - `message_words` - The message words to hash.
    pub fn hash(
        &self,
        layouter: &mut impl Layouter<Fp>,
        message_words: &[AssignedCell<Fp, Fp>; L],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let chip = Pow5Chip::construct(self.config.poseidon_config.clone());

        let message = layouter.assign_region(
            || "load message words",
            |mut region| {
                let result = message_words
                    .iter()
                    .enumerate()
                    .map(|(i, word)| {
                        word.copy_advice(
                            || format!("word {}", i),
                            &mut region,
                            self.config.inputs[i],
                            0,
                        )
                    })
                    .collect::<Result<Vec<AssignedCell<Fp, Fp>>, Error>>();
                Ok(result?.try_into().unwrap())
            },
        )?;

        let hasher = Hash::<_, _, S, ConstantLength<L>, WIDTH, RATE>::init(
            chip,
            layouter.namespace(|| "hasher init"),
        )?;
        hasher.hash(layouter.namespace(|| "hash"), message)
    }
}
