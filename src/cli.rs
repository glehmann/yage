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

    #[arg(long, hide = true)]
    pub markdown_help: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Keygen(KeygenArgs),
    Pubkey(PubkeyArgs),
    Edit(EditArgs),
    Encrypt(EncryptArgs),
    Decrypt(DecryptArgs),
    Env(EnvArgs),
}

/// Generate a new age key
///
/// The public part of the key is logged to the standard error output. It may be computed from the private
/// key with the pubkey command.
///
/// The key is written in the age format, which is compatible with the age tool.
#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// The output path to the private key file
    ///
    /// The private key is written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,
}

/// Convert private keys to their public key
#[derive(Args, Debug)]
pub struct PubkeyArgs {
    /// Decrypt with the specified key
    #[clap(env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key at PATH
    #[clap(short = 'K', long = "key-file", name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_files: Vec<PathBuf>,

    /// The output path to the private key file
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,
}

/// Edit an encrypted YAML file
#[derive(Args, Debug)]
pub struct EditArgs {
    /// Decrypt with the specified key
    #[clap(short, long = "key", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key at PATH
    #[clap(short = 'K', long = "key-file", name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_files: Vec<PathBuf>,

    /// Encrypt to the specified recipients
    #[clap(short, long = "recipient", env = "YAGE_RECIPIENT")]
    pub recipients: Vec<String>,

    /// Encrypt to recipients listed at PATH
    #[clap(short = 'R', long = "recipient-file", env = "YAGE_RECIPIENT_FILE")]
    pub recipient_files: Vec<PathBuf>,

    /// The editor command to use
    #[clap(short, long, env = "EDITOR")]
    pub editor: PathBuf,

    /// The YAML file to decrypt
    #[arg()]
    pub file: PathBuf,
}

/// Encrypted YAML file
#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// Encrypt to the specified recipients
    #[clap(short, long = "recipient", env = "YAGE_RECIPIENT")]
    pub recipients: Vec<String>,

    /// Encrypt to recipients listed at PATH
    #[clap(
        short = 'R',
        long = "recipient-file",
        name = "PATH",
        env = "YAGE_RECIPIENT_FILE"
    )]
    pub recipients_files: Vec<PathBuf>,

    /// Encrypt in place
    #[clap(short, long)]
    pub in_place: bool,

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
    #[clap(short, long = "key", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key at PATH
    #[clap(short = 'K', long = "key-file", name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_files: Vec<PathBuf>,

    /// Decrypt in place
    #[clap(short, long)]
    pub in_place: bool,

    /// The output path to the decrypted YAML file
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The YAML file to decrypt
    #[arg()]
    pub file: PathBuf,
}

/// Execute a command with decrypted values inserted into the environment
#[derive(Args, Debug)]
pub struct EnvArgs {
    /// Decrypt with the specified key
    #[clap(short, long = "keyà  ", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key at PATH
    #[clap(short = 'K', long, name = "PATH", env = "YAGE_KEY_FILE")]
    pub key_files: Vec<PathBuf>,

    /// Start with an empty environment
    #[clap(short, long, default_value_t = false)]
    pub ignore_environment: bool,

    /// The YAML file to decrypt
    #[arg()]
    pub file: PathBuf,

    /// Command to run
    #[arg()]
    pub command: String,

    /// Command arguments
    #[arg()]
    pub args: Vec<String>,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
