use crypto::signature::blind_sign;

/// This configurably defines what underlying primitive type will be used for the candidate ID.
pub type CandidateId = u8;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Authority {
    pub addr: String,
    pub authority_key: blind_sign::PublicKey,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Candidate {
    pub name: String,
    pub id: CandidateId,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BlockchainConfig {
    pub authorities: Vec<Authority>,
    pub candidates: Vec<Candidate>,
}
