#[macro_use]
extern crate log;

use std::io;

use clap::CommandFactory;
use clap::Parser;

use yage::cli;
use yage::cmd;
use yage::error;

fn run() -> error::Result<i32> {
    let cli = cli::Cli::parse();
    if let Some(level) = cli.verbose.log_level() {
        ocli::init(level).unwrap();
    }

    if let Some(shell) = cli.completion {
        clap_complete::generate(shell, &mut cli::Cli::command(), "yage", &mut io::stdout());
        return Ok(0);
    }

    match &cli.command.unwrap() {
        cli::Commands::Keygen(args) => cmd::keygen(args),
        cli::Commands::Pubkey(args) => cmd::pubkey(args),
        cli::Commands::Edit(args) => cmd::edit(args),
        cli::Commands::Encrypt(args) => cmd::encrypt(args),
        cli::Commands::Decrypt(args) => cmd::decrypt(args),
        cli::Commands::Env(args) => cmd::env(args),
        cli::Commands::Check(args) => cmd::check(args),
        cli::Commands::Recipients(args) => cmd::recipients(args),
        cli::Commands::ReEncrypt(args) => cmd::re_encrypt(args),
    }
}

fn main() {
    match run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(err) => {
            error!("{err}");
            std::process::exit(1);
        }
    }
}
