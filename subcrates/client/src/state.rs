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
use leptos::{with, RwSignal, SignalSet, SignalWith};
use protocol::{self, candidate_id::CandidateId, vote::Vote};

// TODO Add proper documentation when the client's logic is more stable.
// TODO Figure out how to display user friendly errors.
// TODO Ensure that keys cannot be read from garbage after user had logged out.

// Helper macro to clean up code around the `with!` Leptos macro:
macro_rules! apply_read_only {
    ( $($var:ident),* ) => {
        $(
            let $var = $var.read_only();
        )*
    };
}

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
    candidate: RwSignal<Option<CandidateId>>,
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
        let signer_pub_key = self
            .signer
            .with(|signer| signer.as_ref().map(digital_sign::Signer::get_public_key))
            .ok_or(anyhow!("User is not logged in"))?;
        let blinder = blind_sign::Blinder::new(authority_key.clone())?;
        let (blinded_pk, unblinder) = blinder.blind(&signer_pub_key)?;
        self.authority_key.set(Some(authority_key));
        self.unblinder.set(Some(unblinder));
        self.blinded_pk.set(Some(blinded_pk));
        self.save()?;

        Ok(())
    }

    pub fn get_blinded_pk(&self) -> RwSignal<Option<blind_sign::BlindedMessage>> {
        self.blinded_pk
    }

    pub fn unblind(&mut self, blind_signature: blind_sign::BlindSignature) -> Result<()> {
        let Self {
            signer, unblinder, ..
        } = self;
        apply_read_only!(signer, unblinder);
        let access_token = with!(|signer, unblinder| {
            match (signer, unblinder) {
                (Some(signer), Some(unblinder)) => unblinder
                    .unblind_signature(blind_signature, &signer.get_public_key())
                    .map_err(std::convert::Into::into),
                _ => Err(anyhow!("State is corrupted")),
            }
        })?;
        self.access_token.set(Some(access_token));
        self.save()?;

        Ok(())
    }

    pub fn vote(&mut self, candidate: &str, _blockchain_addr: &str) -> Result<()> {
        let candidate = CandidateId::new(candidate.parse()?);
        let Self {
            signer,
            access_token,
            ..
        } = self;
        apply_read_only!(signer, access_token);

        let _vote = with!(|signer, access_token| {
            match (signer, access_token) {
                (Some(signer), Some(access_token)) => {
                    Vote::new(signer, candidate.clone(), chrono::Utc::now(), access_token)
                        .map_err(std::convert::Into::into)
                }
                _ => Err(anyhow!("State is corrupted")),
            }
        })?;

        self.candidate.set(Some(candidate.clone()));
        self.save()?;

        Ok(())
    }

    fn save(&self) -> Result<()> {
        let Self {
            encryption,
            signer,
            authority_key,
            unblinder,
            access_token,
            candidate,
            ..
        } = self;
        apply_read_only!(signer, authority_key, unblinder, access_token, candidate);
        let key_store = with!(
            |signer, authority_key, unblinder, access_token, candidate| {
                KeyStore {
                    signer_sk: signer
                        .as_ref()
                        .map(|signer| signer.get_secret_key().clone()),
                    authority_key: authority_key.clone(),
                    unblinding_secret: unblinder
                        .as_ref()
                        .map(|unblinder| unblinder.get_unblinding_secret().clone()),
                    access_token: access_token.clone(),
                    candidate: candidate.clone(),
                }
            }
        );
        encryption
            .with(|encryption| {
                encryption
                    .as_ref()
                    .map(|encryption| key_store.encrypt(encryption))
            })
            .ok_or(anyhow!("User is not logged in"))??
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
        let candidate = "5";
        let blockchain_addr = "www.blockchain.com";

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

        state.vote(candidate, blockchain_addr).unwrap();
        assert!(matches!(state.get_status(), Status::Voted));
    }
}
