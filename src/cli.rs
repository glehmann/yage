use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::cmd;

/// A simple tool to manage encrypted secrets in YAML files with age encryption
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Generate the completion code for this shell
    #[arg(long, value_name = "SHELL")]
    pub completion: Option<clap_complete::Shell>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Check(cmd::CheckArgs),
    Decrypt(cmd::DecryptArgs),
    Edit(cmd::EditArgs),
    Encrypt(cmd::EncryptArgs),
    Env(cmd::EnvArgs),
    Keygen(cmd::KeygenArgs),
    Pubkey(cmd::PubkeyArgs),
    Recipients(cmd::RecipientsArgs),
    ReEncrypt(cmd::ReEncryptArgs),
}

#[cfg(windows)]
pub const ENV_PATH_SEP: char = ';';
#[cfg(not(windows))]
pub const ENV_PATH_SEP: char = ':';

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
