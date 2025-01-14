use protocol::config::BlockchainConfig;

pub struct State {
    blockchain_config: BlockchainConfig,
}

impl State {
    pub fn new(blockchain_config: BlockchainConfig) -> Self {
        Self { blockchain_config }
    }

    pub fn get_blockchain_config(&self) -> &BlockchainConfig {
        &self.blockchain_config
    }
}
