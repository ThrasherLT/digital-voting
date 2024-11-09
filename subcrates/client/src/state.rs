//! This file is for the global state and all the main logic of the client.
//! Altho Leptos recomments using local signals for the logic, instead of a global state,
//! the code, in that case, becomes unacceptably complicated.
//! Besides that, this way the logic can be tested with simple unit tests at the end of this file.

use crate::storage::{KeyStore, Storage};
use anyhow::{anyhow, Result};
use crypto::{
    encryption::symmetric,
    signature::{blind_sign, digital_sign},
};
use leptos::{RwSignal, SignalGet, SignalSet, SignalWith};

// TODO Add proper documentation when the client's logic is more stable.
// TODO Figure out how to display user friendly errors.
// TODO Ensure that keys cannot be read from garbage after user had logged out.

/// Status of the client.
/// Used to select which part of the UI to show to the user.
pub enum Status {
    LoggedOut,
    LoggedIn,
    Validated,
    Voted,
}

// All members of the `State` struct must be reactive signals otherwise they
// won't get loaded when Leptos retrieves them from context.
/// The global state of the client.
#[derive(Clone, Default)]
pub struct State {
    encryption: RwSignal<Option<symmetric::Encryption>>,
    signer: RwSignal<Option<digital_sign::Signer>>,
    authority_key: RwSignal<Option<blind_sign::PublicKey>>,
    blinded_pk: RwSignal<Option<blind_sign::BlindedMessage>>,
    unblinder: RwSignal<Option<blind_sign::Unblinder>>,
    access_token: RwSignal<Option<blind_sign::Signature>>,
    candidate: RwSignal<Option<u8>>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_user(&mut self, username: &str, password: &str) -> Result<()> {
        let encryption = symmetric::Encryption::new(username.as_bytes(), password.as_bytes())?;
        let signer = digital_sign::Signer::new()?;

        KeyStore {
            signer_sk: Some(signer.get_secret_key().to_owned()),
            authority_key: None,
            unblinding_secret: None,
            access_token: None,
            candidate: None,
        }
        .encrypt(&encryption)?
        .save();

        self.encryption.set(Some(encryption));
        self.signer.set(Some(signer));

        Ok(())
    }

    pub fn can_login() -> bool {
        Storage::load().is_some()
    }

    pub fn login_user(&mut self, username: &str, password: &str) -> Result<()> {
        let storage = Storage::load().ok_or(anyhow!("User data is empty"))?;
        let encryption = symmetric::Encryption::load(
            username.as_bytes(),
            password.as_bytes(),
            storage.get_metadata(),
        )?;
        let key_store = storage.decrypt(&encryption)?;
        let signer = if let Some(signer_sk) = key_store.signer_sk {
            Some(digital_sign::Signer::from_secret_key(signer_sk)?)
        } else {
            None
        };
        let unblinder = if let (Some(authority_key), Some(unblinding_secret)) =
            (&key_store.authority_key, key_store.unblinding_secret)
        {
            Some(blind_sign::Unblinder::from_pk_and_secret(
                authority_key.to_owned(),
                unblinding_secret,
            )?)
        } else {
            None
        };

        self.encryption.set(Some(encryption));
        self.signer.set(signer);
        self.authority_key.set(key_store.authority_key);
        // Upon logging in, it is assumed that the user will press the blinding button again to display the
        // blinded pk, if he needs it:
        self.blinded_pk.set(None);
        self.unblinder.set(unblinder);
        self.access_token.set(key_store.access_token);
        self.candidate.set(key_store.candidate);

        Ok(())
    }

    pub fn get_status(&self) -> Status {
        if self.candidate.with(Option::is_some) {
            Status::Voted
        } else if self.access_token.with(Option::is_some) {
            Status::Validated
        } else if self.signer.with(Option::is_some) {
            Status::LoggedIn
        } else {
            Status::LoggedOut
        }
    }

    pub fn blind(&mut self, authority_key: blind_sign::PublicKey) -> Result<()> {
        let (signer_secret_key, signer_pub_key) = self
            .signer
            .with(|signer| {
                signer
                    .as_ref()
                    .map(|signer| (signer.get_secret_key().to_owned(), signer.get_public_key()))
            })
            .ok_or(anyhow!("User is not logged in"))?;
        let blinder = blind_sign::Blinder::new(authority_key.clone())?;
        let (blinded_pk, unblinder) = blinder.blind(&signer_pub_key)?;
        let unblinding_secret = Some(unblinder.get_unblinding_secret());
        self.authority_key.set(Some(authority_key));
        self.unblinder.set(Some(unblinder));
        self.blinded_pk.set(Some(blinded_pk));
        self.save(KeyStore {
            signer_sk: Some(signer_secret_key),
            authority_key: self.authority_key.clone().get(),
            unblinding_secret,
            access_token: None,
            candidate: None,
        })?;

        Ok(())
    }

    pub fn get_blinded_pk(&self) -> RwSignal<Option<blind_sign::BlindedMessage>> {
        self.blinded_pk
    }

    // TODO Maybe it's possible to write this method in a cleaner way?
    pub fn unblind(&mut self, blind_signature: blind_sign::BlindSignature) -> Result<()> {
        let (signer_secret_key, signer_pub_key) = self
            .signer
            .with(|signer| {
                signer
                    .as_ref()
                    .map(|signer| (signer.get_secret_key().to_owned(), signer.get_public_key()))
            })
            .ok_or(anyhow!("User is not logged in"))?;

        let (access_token, unblinding_secret) = self.unblinder.with(|unblinder| {
            unblinder
                .as_ref()
                .map(|unblinder| {
                    (
                        unblinder.unblind_signature(blind_signature, &signer_pub_key),
                        unblinder.get_unblinding_secret(),
                    )
                })
                .map_or((None, None), |(access_token, unblinding_secret)| {
                    (Some(access_token), Some(unblinding_secret))
                })
        });
        let access_token = if let Some(access_token) = access_token {
            Some(access_token?)
        } else {
            None
        };

        self.save(KeyStore {
            signer_sk: Some(signer_secret_key),
            authority_key: self.authority_key.clone().get(),
            unblinding_secret,
            access_token,
            candidate: None,
        })?;

        Ok(())
    }

    pub fn _vote(&mut self, candidate: u8) -> Result<()> {
        self.candidate.set(Some(candidate));
        // TODO Add logic to send the vote to the blockchain here.

        let unblinding_secret = self.unblinder.with(|unblinder| {
            unblinder
                .as_ref()
                .map(blind_sign::Unblinder::get_unblinding_secret)
        });
        self.save(KeyStore {
            signer_sk: self.signer.with(|signer| {
                signer
                    .as_ref()
                    .map(|signer| signer.get_secret_key().to_owned())
            }),
            authority_key: self.authority_key.clone().get(),
            unblinding_secret,
            access_token: self.access_token.clone().get(),
            candidate: Some(candidate),
        })?;

        Ok(())
    }

    fn save(&self, keystore: KeyStore) -> Result<()> {
        self.encryption
            .with(|encryption| {
                encryption
                    .as_ref()
                    .map(|encryption| keystore.encrypt(encryption))
            })
            .ok_or(anyhow!("User not logged in"))??
            .save();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod tests {
    use super::*;

    fn logout_login(_state: State, username: &str, password: &str) -> State {
        let mut state = State::new();
        state.login_user(username, password).unwrap();

        state
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    #[test]
    // State contains Leptos signals which can only be used on WASM targets.
    fn test_state() {
        let username = "Admin";
        let password = "Password";
        let candidate = 5;

        let authority_signer = blind_sign::BlindSigner::new().unwrap();

        let mut state = State::new();
        assert!(matches!(state.get_status(), Status::LoggedOut));

        assert!(!State::can_login());
        state.register_user(username, password).unwrap();
        assert!(matches!(state.get_status(), Status::LoggedIn));
        assert!(State::can_login());
        let mut state = logout_login(state, username, password);
        assert!(matches!(state.get_status(), Status::LoggedIn));

        state
            .blind(authority_signer.get_public_key().unwrap())
            .unwrap();
        // User is expected to press blinding button upon login at this point, so no need to test relogin here.

        let blind_signature = authority_signer
            .bling_sign(&state.blinded_pk.get().unwrap())
            .unwrap();

        state.unblind(blind_signature).unwrap();
        let mut state = logout_login(state, username, password);
        assert!(matches!(state.get_status(), Status::Validated));

        state._vote(candidate).unwrap();
        assert!(matches!(state.get_status(), Status::Voted));
    }
}
