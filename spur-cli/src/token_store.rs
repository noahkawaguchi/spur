use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use std::{fs, path::PathBuf};

/// Saves the token to a text file.
pub fn save(token: &str) -> Result<()> {
    fs::write(get_file_path()?, token).context("failed to write token to file".red())
}

/// Reads the saved token if it exists.
pub fn load() -> Result<String> {
    fs::read_to_string(get_file_path()?).context("failed to read the token from file".red())
}

/// Gets the path to the token file, e.g. `~/.spur/token.txt`, creating the `.spur` directory if it
/// doesn't exist.
fn get_file_path() -> Result<PathBuf> {
    let dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow!("could not find home directory".red()))?
        .join(".spur");

    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create directories for path {dir:?}").red())?;

    Ok(dir.join("token.txt"))
}
