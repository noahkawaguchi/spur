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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::{
        TempDir,
        assert::PathAssert,
        prelude::{FileWriteStr, PathChild},
    };

    #[test]
    fn saves_token_correctly() {
        let temp_home = TempDir::new().expect("failed to create temp home");
        let store = LocalTokenStore::new(&temp_home).expect("failed to initialize store");

        let token = "my token";
        store.save(token).expect("failed to save token");

        temp_home.child(".spur").child("token.txt").assert(token);
        temp_home.close().expect("failed to close temp home");
    }

    #[test]
    fn loads_token_correctly() {
        let temp_home = TempDir::new().expect("failed to create temp home");
        let store = LocalTokenStore::new(&temp_home).expect("failed to initialize store");

        let token = "your token";
        temp_home
            .child(".spur")
            .child("token.txt")
            .write_str(token)
            .expect("failed to write to token file");

        let got_token = store.load().expect("failed to load token");
        assert_eq!(got_token, token);
        temp_home.close().expect("failed to close temp home");
    }
}
