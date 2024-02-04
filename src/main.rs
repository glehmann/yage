#[macro_use]
extern crate log;

use std::io;

use clap::CommandFactory;
use clap::Parser;

use yage::cli;
use yage::cmd;
use yage::error;

fn run() -> error::Result<()> {
    let cli = cli::Cli::parse();
    if let Some(level) = cli.verbose.log_level() {
        ocli::init(level).unwrap();
    }

    if let Some(shell) = cli.completion {
        clap_complete::generate(shell, &mut cli::Cli::command(), "yage", &mut io::stdout());
        return Ok(());
    }

    match &cli.command {
        cli::Commands::Keygen(args) => cmd::keygen(args)?,
        cli::Commands::Pubkey(args) => cmd::pubkey(args)?,
        cli::Commands::Edit(args) => cmd::edit(args)?,
        cli::Commands::Encrypt(args) => cmd::encrypt(args)?,
        cli::Commands::Decrypt(args) => cmd::decrypt(args)?,
        cli::Commands::Env(args) => cmd::env(args)?,
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}
