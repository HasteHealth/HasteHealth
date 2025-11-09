use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    Transaction {},
}
