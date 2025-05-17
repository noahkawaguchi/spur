use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new account")]
    Signup,

    #[command(about = "Log in to an existing account")]
    Login,

    #[command(about = "Confirm JWT validity")]
    Check,

    #[command(about = "Add a friend by username")]
    Add { username: String },
}
