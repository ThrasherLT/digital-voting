use crate::json_base64::serde_base64_json;
use crate::Timestamp;
use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UnparsedVote {
    #[serde(with = "serde_base64_json")]
    pub pkey: Vec<u8>,
    pub vote: u32,
    pub timestamp: Timestamp,
    #[serde(with = "serde_base64_json")]
    pub signature: Vec<u8>,
}
