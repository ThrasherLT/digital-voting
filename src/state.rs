use protocol::config::ElectionConfig;

pub struct State {
    election_config: ElectionConfig,
}

impl State {
    #[must_use]
    pub fn new(election_config: ElectionConfig) -> Self {
        Self { election_config }
    }

    #[must_use]
    pub fn get_election_config(&self) -> &ElectionConfig {
        &self.election_config
    }
}
