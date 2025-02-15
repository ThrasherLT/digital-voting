use std::net::SocketAddr;

use crypto::signature::blind_sign;

use crate::timestamp::Timestamp;

/// This configurably defines what underlying primitive type will be used for the candidate ID.
pub type CandidateId = u8;

/// The structure of the blockchain config. Used to prepare the nodes and clients for interoperation.
#[allow(clippy::module_name_repetitions)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ElectionConfig {
    /// The name of ID of the election. Used to differentiate elections on this blockchain.
    pub name: String,
    /// Beginning of the election. No votes can be cast before this time.
    pub start: Timestamp,
    /// End of the election. No votes can be cast after this time.
    pub end: Timestamp,
    /// A list of blockchain nodes holding this election.
    pub nodes: Vec<SocketAddr>,
    /// A list of authorities which are validating the voters for this election.
    pub authorities: Vec<Authority>,
    /// A list of candidates participating in this election.
    pub candidates: Vec<Candidate>,
}

/// Election authority which validates that voters are eligible to vote.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Authority {
    /// The address on which the authority is accessible.
    pub addr: String,
    /// The public key of this authority.
    pub authority_key: blind_sign::PublicKey,
}

/// A Candidate participating in an election.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Candidate {
    /// Name of the candidate. Used for display purposes only.
    pub name: String,
    /// Unique ID of the candidate which will be visible on the blockchain.
    pub id: CandidateId,
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test functions more as an example on how to structure the blockchain config in json format.
    #[test]
    fn test_config_parsing_json() {
        let config_in_json_string = r#"
{
  "name": "Trailer Park Supervisor CA, Nova Scotia, Dartmouth, 2025",
  "start": "2025-03-01T00:00:00Z",
  "end": "2025-03-03T12:59:59Z",
  "nodes": [
        "123.123.123.123:12345",
        "123.123.123.123:12344",
        "121.123.123.123:12345",
        "127.0.0.1:12345"
  ],
  "candidates": [
    {
      "name": "Ricky",
      "id": 0
    },
    {
      "name": "Randy",
      "id": 1
    }
  ],
  "authorities": [
    {
      "addr": "http://DESKTOP-B24QLMC:32950",
      "authority_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA7890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGH"
    },
    {
      "addr": "http://DESKTOP-B24QLMC:32951",
      "authority_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA7890ABCDEFGHIJKLMN1234567890BBCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGH"
    },
    {
      "addr": "http://DESKTOP-B24QLMC:32949",
      "authority_key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA7890ABCDEFGHIJKLMN1234567890CBCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGHIJKLMN1234567890ABCDEFGH"
    }
  ]
}
"#;
        serde_json::from_str::<ElectionConfig>(&config_in_json_string).unwrap();
    }
}
