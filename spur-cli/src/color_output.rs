use anyhow::{Error, Result, anyhow};
use colored::{ColoredString, Colorize};

/// Colors the first (or only) line of a success message green and the first (or only) line of an
/// error message red.
pub fn color_first_line(result: Result<String, Error>) -> Result<ColoredString, Error> {
    match result {
        Ok(message) => match message.split_once('\n') {
            Some((first, rest)) => Ok(format!("{}\n{}", first.green(), rest).into()),
            None => Ok(message.green()),
        },
        Err(e) => {
            let err = e.to_string();
            match err.split_once('\n') {
                Some((first, rest)) => Err(anyhow!(format!("{}\n{}", first.red(), rest))),
                None => Err(anyhow!(err.red())),
            }
        }
    }
}
