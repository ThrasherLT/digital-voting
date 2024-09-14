pub mod serde_base64_json {
    use base64::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = BASE64_STANDARD.encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        BASE64_STANDARD
            .decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct UnparsedVote {
        #[serde(with = "serde_base64_json")]
        pub pkey: Vec<u8>,
        pub vote: u32,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        #[serde(with = "serde_base64_json")]
        pub signature: Vec<u8>,
    }

    #[actix_web::test]
    async fn test_deserialize_base64() {
        let unparsed_vote = UnparsedVote {
            pkey: vec![0, 1, 2, 3],
            vote: 1,
            timestamp: chrono::Utc::now(),
            signature: vec![4, 5, 6, 7],
        };
        let serialized_json_vote = serde_json::to_string(&unparsed_vote).unwrap();
        let deserialized_vote: UnparsedVote = serde_json::from_str(&serialized_json_vote).unwrap();
        assert_eq!(unparsed_vote, deserialized_vote);
    }
}
