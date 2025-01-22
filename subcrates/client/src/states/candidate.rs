use anyhow::Result;

use protocol::config::CandidateId;

use crate::{states::user::User, storage::Storage};

// TODO Add timestamp to candidate.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Candidate(pub CandidateId);

impl Candidate {
    fn storage_key(username: &str, blockchain: &str) -> String {
        format!("{}/{}/candidate", username, blockchain)
    }

    pub fn choose(candidate: CandidateId, user: &User, blockchain: &str) -> Result<Self> {
        let candidate = Candidate(candidate);

        Storage::encrypt(&user.encryption, &candidate)?
            .save(&Self::storage_key(&user.username, blockchain));
        Ok(candidate)
    }

    pub fn load(user: &User, blockchain: &str) -> Result<Option<Self>> {
        match Storage::load(&Self::storage_key(&user.username, blockchain)) {
            Some(candidate_storage) => {
                let candidate: Self = candidate_storage.decrypt(&user.encryption)?;
                Ok(Some(candidate))
            }
            None => Ok(None),
        }
    }

    pub fn delete(user: &User, blockchain: &str) {
        Storage::delete(&Self::storage_key(&user.username, blockchain));
    }
}
