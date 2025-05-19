use anyhow::{Context, Result};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub trait TokenStore: Send + Sync {
    /// Saves the token to a text file in the app directory, using `0o600` (`-rw-------`)
    /// permissions if on Unix.
    fn save(&self, token: &str) -> Result<()>;
    /// Reads the saved token if it exists.
    fn load(&self) -> Result<String>;
}

pub struct LocalTokenStore {
    token_path: PathBuf,
}

impl LocalTokenStore {
    /// Creates the `.spur` app directory in the user's home directory if it doesn't exist. If on
    /// Unix, sets `0o700` (`drwx------`) permissions (even if the directory already existed).
    pub fn new(home_dir: &Path) -> Result<Self> {
        let app_dir = home_dir.join(".spur");

        fs::create_dir_all(&app_dir)
            .with_context(|| format!("failed to create app directory at {app_dir:?}"))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(&app_dir, perms).with_context(|| {
                format!("failed to set secure Unix permissions for app directory at {app_dir:?}")
            })?;
        }

        Ok(Self { token_path: app_dir.join("token.txt") })
    }
}

impl TokenStore for LocalTokenStore {
    fn save(&self, token: &str) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.token_path)
            .context("failed to open file for writing")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .context("failed to set secure Unix permissions for token file")?;
        }

        file.write_all(token.as_bytes())
            .context("failed to write token to file")
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
