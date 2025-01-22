use anyhow::Result;

use protocol::config::BlockchainConfig;

use super::{
    access_tokens::AccessTokens, config::Config, signature::Signature, user::User,
    validators::Validators,
};

pub fn delete_from_storage(blockchain_addr: &str, user: &mut User) {
    Signature::delete(&user, blockchain_addr);
    Config::delete(&user, blockchain_addr);
    Validators::delete(&user, blockchain_addr);
    AccessTokens::delete(&user, blockchain_addr);
    // TODO Delete candidate too.
}

pub fn create_in_storage(
    blockchain_addr: String,
    user: &mut User,
    blockchain_config: BlockchainConfig,
) -> Result<()> {
    let signature = Signature::new(&user, &blockchain_addr)?;
    Validators::new(
        &blockchain_config,
        signature.signer.get_public_key(),
        &user,
        &blockchain_addr,
    )?;
    let _ = AccessTokens::new(&user, &blockchain_addr, blockchain_config.authorities.len())?;
    Config::save(blockchain_config, &user, &blockchain_addr)
    // TODO Add candidate too.
}
