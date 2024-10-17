use crate::Timestamp;
use serde::{self, Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UnparsedVote {
    #[serde_as(as = "Base64")]
    pub pkey: Vec<u8>,
    pub vote: u32,
    pub timestamp: Timestamp,
    #[serde_as(as = "Base64")]
    pub signature: Vec<u8>,
}
