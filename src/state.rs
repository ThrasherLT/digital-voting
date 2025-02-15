use protocol::config::ElectionConfig;

pub struct State {
    blockchain_config: ElectionConfig,
}

impl State {
    #[must_use]
    pub fn new(blockchain_config: ElectionConfig) -> Self {
        Self { blockchain_config }
    }

    #[must_use]
    pub fn get_blockchain_config(&self) -> &ElectionConfig {
        &self.blockchain_config
    }
}
