use crate::auth::TokenStore;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct LocalTokenStore {
    token_path: PathBuf,
}

impl LocalTokenStore {
    /// Creates the .spur directory if it doesn't exist.
    pub fn new(home_dir: &Path) -> Result<Self> {
        let app_dir = home_dir.join(".spur");

        fs::create_dir_all(&app_dir)
            .with_context(|| format!("failed to create app directory at {app_dir:?}"))?;

        Ok(Self { token_path: app_dir.join("token.txt") })
    }
}

impl TokenStore for LocalTokenStore {
    fn save(&self, token: &str) -> Result<()> {
        fs::write(&self.token_path, token).context("failed to write token to file")
    }

    fn load(&self) -> Result<String> {
        fs::read_to_string(&self.token_path).context("failed to read token from file")
    }
}
