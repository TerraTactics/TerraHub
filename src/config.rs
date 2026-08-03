//! Hub configuration loaded from TOML.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HubConfig {
    pub hub: HubSection,
    pub radio: RadioSection,
    pub buffer: BufferSection,
    pub cloud: CloudSection,
    pub admin: AdminSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HubSection {
    pub identity: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RadioSection {
    pub backend: String,
    #[serde(default)]
    pub device: Option<String>,
    #[serde(default)]
    pub baud: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BufferSection {
    pub sqlite_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloudSection {
    pub enabled: bool,
    #[serde(default)]
    pub broker_url: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminSection {
    pub bind: String,
}

impl HubConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        let config: HubConfig = toml::from_str(&text).context("parse TOML config")?;
        Ok(config)
    }
}
