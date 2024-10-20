//! This module contains helper code to simplify code related to election candidate representation.
//! The main reason for this was to implement a type which could hold primitive values and implement `AsRef<[u8]>` for them.

use std::hash::Hash;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when working with election candidates.
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to parse the candidate value from string.
    #[error("Failed to parse candidate from string: {}", .0)]
    ParseFromString(#[from] std::num::ParseIntError),
}

/// This configurably defines what underlying primitive type will be used to describe the candidate.
type UnderlyingType = u8;

/// The wrapper struct for primitive types to represent election candidates.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone)]
pub struct CandidateId([u8; std::mem::size_of::<UnderlyingType>()]);

impl CandidateId {
    /// Function used to create new primitive candidate from underlying primitive type.
    #[must_use]
    pub fn new(candidate: UnderlyingType) -> Self {
        Self(candidate.to_le_bytes())
    }

    /// Function used to get the underlying primitive type from the primitive candidate.
    #[must_use]
    pub fn get(&self) -> UnderlyingType {
        UnderlyingType::from_le_bytes(self.0)
    }
}

impl AsRef<[u8]> for CandidateId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for CandidateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl std::str::FromStr for CandidateId {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(CandidateId::new(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_vote() {
        let primitive_candidate_original = 1 as usize;

        let primitive_candidate = primitive_candidate_original.to_string();
        let primitive_candidate: usize = primitive_candidate.parse().unwrap();
        assert_eq!(primitive_candidate_original, primitive_candidate);
    }
}
