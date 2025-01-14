use std::path::Path;

use anyhow::Result;
use protocol::config::BlockchainConfig;
use tokio::fs;

pub async fn load_from_file(path: &Path) -> Result<BlockchainConfig> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file_content = fs::read_to_string(path).await?;

    Ok(serde_json::from_str(&file_content)?)
}
