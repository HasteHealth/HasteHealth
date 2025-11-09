use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    Transaction {},
}
