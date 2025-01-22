use protocol::config::BlockchainConfig;

pub struct State {
    blockchain_config: BlockchainConfig,
}

impl State {
    #[must_use]
    pub fn new(blockchain_config: BlockchainConfig) -> Self {
        Self { blockchain_config }
    }

    #[must_use]
    pub fn get_blockchain_config(&self) -> &BlockchainConfig {
        &self.blockchain_config
    }
}
