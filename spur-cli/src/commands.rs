use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    // Auth commands
    #[command(about = "Create a new account")]
    Signup,

    #[command(about = "Log in to an existing account")]
    Login,

    #[command(about = "Confirm JSON Web Token validity")]
    Check,

    // Friendship commands
    #[command(about = "Add a friend by username")]
    Add { username: String },

    #[command(about = "List your friends")]
    Friends,

    #[command(about = "List pending friend requests to you")]
    Requests,

    // Prompt and post commands
    #[command(about = "Create a new prompt")]
    Prompt { body: String },

    #[command(about = "Create a new post in response to a prompt")]
    Write { prompt_id: i32 },

    #[command(about = "Read a friend's post")]
    Read { post_id: i32 },

    #[command(about = "List a friend's prompts and posts")]
    Profile { username: String },

    #[command(about = "List your own prompts and posts")]
    Me,

    #[command(about = "List prompts and posts from all your friends")]
    Feed,
}
