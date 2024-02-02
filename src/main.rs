#[macro_use]
extern crate log;

use clap::Parser;
use cli::Commands;

mod cli;
mod decrypt;
mod edit;
mod encrypt;
mod env;
mod error;
mod keygen;
mod pubkey;
mod util;

fn run() -> error::Result<()> {
    let cli = cli::Cli::parse();
    if let Some(level) = cli.verbose.log_level() {
        ocli::init(level).unwrap();
    }

    if cli.markdown_help {
        clap_markdown::print_help_markdown::<cli::Cli>();
        return Ok(());
    }

    match cli.command.unwrap() {
        Commands::Keygen(ref args) => keygen::keygen(args)?,
        Commands::Pubkey(ref args) => pubkey::pubkey(args)?,
        Commands::Edit(ref args) => edit::edit(args)?,
        Commands::Encrypt(ref args) => encrypt::encrypt(args)?,
        Commands::Decrypt(ref args) => decrypt::decrypt(args)?,
        Commands::Env(ref args) => env::env(args)?,
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}
