use anyhow::{anyhow, Result};

use crypto::signature::digital_sign;

use crate::{states::user::User, storage::Storage};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Signature {
    #[serde(with = "signer_serde")]
    pub signer: digital_sign::Signer,
}

mod signer_serde {
    use crypto::signature::digital_sign;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(signer: &digital_sign::Signer, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        signer.get_secret_key().clone().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<digital_sign::Signer, D::Error>
    where
        D: Deserializer<'de>,
    {
        let signer_sk = digital_sign::SecretKey::deserialize(deserializer)?;
        digital_sign::Signer::from_secret_key(signer_sk).map_err(de::Error::custom)
    }
}

impl Signature {
    fn storage_key(username: &str, blockchain: &str) -> String {
        format!("{}/{}/signature", username, blockchain)
    }

    pub fn new(user: &User, blockchain: &str) -> Result<Self> {
        let signer = digital_sign::Signer::new()?;

        let signature = Signature { signer };
        Storage::encrypt(&user.encryption, &signature)?
            .save(&Self::storage_key(&user.username, blockchain));

        Ok(signature)
    }

    pub fn load(user: &User, blockchain: &str) -> Result<Self> {
        let signature_storage = Storage::load(&Self::storage_key(&user.username, blockchain))
            .ok_or(anyhow!("User or password are incorrect"))?;
        let signature: Self = signature_storage.decrypt(&user.encryption)?;

        Ok(signature)
    }

    pub fn delete(user: &User, blockchain: &str) {
        Storage::delete(&Self::storage_key(&user.username, blockchain));
    }
}
