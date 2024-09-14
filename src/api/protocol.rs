use crate::json_base64::json_base64_ser;
use crate::Timestamp;
use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UnparsedVote {
    #[serde(with = "json_base64_ser")]
    pub pkey: Vec<u8>,
    pub vote: u32,
    pub timestamp: Timestamp,
    #[serde(with = "json_base64_ser")]
    pub signature: Vec<u8>,
}
