#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod commands;

use clap::Parser;
use commands::{
    Cli,
    Commands::{Check, Login, Signup},
};

fn main() {
    match Cli::parse().command {
        Signup => println!("signup used!"),
        Login => println!("login used!"),
        Check => println!("check used!"),
    }
}
