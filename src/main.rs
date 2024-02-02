#[macro_use]
extern crate log;

use std::io;

use clap::CommandFactory;
use clap::Parser;

use yage::cli;
use yage::decrypt;
use yage::edit;
use yage::encrypt;
use yage::env;
use yage::error;
use yage::keygen;
use yage::pubkey;

fn run() -> error::Result<()> {
    let cli = cli::Cli::parse();
    if let Some(level) = cli.verbose.log_level() {
        ocli::init(level).unwrap();
    }

    if let Some(shell) = cli.completion {
        clap_complete::generate(shell, &mut cli::Cli::command(), "yage", &mut io::stdout());
        return Ok(());
    }

    if cli.markdown_help {
        clap_markdown::print_help_markdown::<cli::Cli>();
        return Ok(());
    }

    match cli.command.unwrap() {
        cli::Commands::Keygen(ref args) => keygen::keygen(args)?,
        cli::Commands::Pubkey(ref args) => pubkey::pubkey(args)?,
        cli::Commands::Edit(ref args) => edit::edit(args)?,
        cli::Commands::Encrypt(ref args) => encrypt::encrypt(args)?,
        cli::Commands::Decrypt(ref args) => decrypt::decrypt(args)?,
        cli::Commands::Env(ref args) => env::env(args)?,
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}
