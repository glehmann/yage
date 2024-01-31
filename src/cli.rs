use std::path::PathBuf;

use clap::{command, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

/// A simple tool to manage encrypted secrets in YAML files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Keygen(KeygenArgs),
    Edit(EditArgs),
    Encrypt(EncryptArgs),
    Decrypt(DecryptArgs),
    Env(EnvArgs),
}

/// Generate a new key
#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// The output path to the private key file
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,
}

/// Edit an encrypted YAML file
#[derive(Args, Debug)]
pub struct EditArgs {
    /// The private key
    #[clap(short, long, env = "YAGE_KEY")]
    pub key: Option<String>,

    /// The path to the private key file
    #[clap(short = 'K', long, name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_file: Option<PathBuf>,
}

/// Encrypted YAML file
#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// Encrypt to the specified recipients
    #[clap(short, long = "recipient", env = "YAGE_RECIPIENTS")]
    pub recipients: Vec<String>,

    /// Encrypt to recipients listed at PATH
    #[clap(
        short = 'R',
        long = "recipient-path",
        name = "PATH",
        env = "YAGE_RECIPIENTS_FILE"
    )]
    pub recipients_paths: Vec<PathBuf>,

    /// Encrypt in place
    #[clap(short, long)]
    pub inplace: bool,

    /// The output path to the encrypted YAML file
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The YAML file to encrypt
    #[arg()]
    pub file: PathBuf,
}

/// Decrypted YAML file
#[derive(Args, Debug)]
pub struct DecryptArgs {
    /// Decrypt with the specified key
    #[clap(short, long, env = "YAGE_KEY")]
    pub key: Option<String>,

    /// Decrypt with the key at PATH
    #[clap(short = 'K', long, name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_file: Option<PathBuf>,

    /// Decrypt in place
    #[clap(short, long)]
    pub inplace: bool,
}

/// Execute a command with decrypted values inserted into the environment
#[derive(Args, Debug)]
pub struct EnvArgs {
    /// Decrypt with the specified key
    #[clap(short, long, env = "YAGE_KEY")]
    pub key: Option<String>,

    /// Decrypt with the key at PATH
    #[clap(short = 'K', long, name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_file: Option<PathBuf>,

    /// Start with an empty environment
    #[clap(short, long, default_value_t = false)]
    pub ignore_environment: bool,

    /// Command to run
    #[arg(name = "COMMAND")]
    pub args: Vec<String>,
}
