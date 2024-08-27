//! This module contains the implementation of the set membership zk proof system, Merkle tree
//! and some other useful functionality required to verify that the voter ID is among the registered
//! voter ID's without disclosing which voter ID it is.

#![deny(missing_docs)]

// TODO doctests are forcing me to make some mods pub, so need to investigate how to keep them private.
pub mod merkle;
mod poseidon_chip;
pub mod poseidon_hasher;
pub mod set_membership;
pub mod set_membership_circuit;
