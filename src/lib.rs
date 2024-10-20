// TODO
#![allow(clippy::missing_errors_doc)]

use std::{collections::HashMap, fmt::Display};

use thiserror::Error;

pub mod api;

pub mod batcher;
pub mod logging;

mod blockchain;
use blockchain::{BlockValue, Blockchain, Error as BlockchainError};
use protocol::{candidate_id::CandidateId, vote::Vote};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Votes tampered with")]
    VotesTampered,
    #[error(transparent)]
    BlockchainError(#[from] BlockchainError),
    #[error("Unknown error")]
    Unknown,
}

impl BlockValue for Vote {}

#[derive(Debug)]
pub struct VotingSystem {
    blockchain: Blockchain<Vote>,
}

impl VotingSystem {
    #[must_use]
    pub fn new() -> Self {
        Self {
            blockchain: Blockchain::new(),
        }
    }

    pub fn add_votes(mut self, votes: Vec<Vote>) -> Result<Self, Error> {
        self.blockchain.add_block(votes)?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.blockchain.validate_hashes().is_ok() {
            Ok(())
        } else {
            Err(Error::VotesTampered)
        }
    }

    pub fn tally_votes(&self) -> Result<Tally, Error> {
        let mut tally = HashMap::new();

        self.blockchain.iter().for_each(|values| {
            for vote in values {
                let count = tally.entry(vote.get_candidate().clone()).or_insert(0);
                *count += 1;
            }
        });
        Ok(Tally(tally))
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Error> {
        self.blockchain.save_to_file(filename)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Error> {
        let blockchain = Blockchain::load_from_file(filename)?;
        Ok(Self { blockchain })
    }
}

impl Default for VotingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for VotingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.blockchain)
    }
}

#[derive(PartialEq, Debug)]
pub struct Tally(HashMap<CandidateId, u64>);

impl Display for Tally {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (candidate_id, count) in &self.0 {
            writeln!(f, "Candidate {candidate_id} has {count} votes")?;
        }
        Ok(())
    }
}
