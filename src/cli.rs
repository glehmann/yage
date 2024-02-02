use std::path::PathBuf;

use clap::{command, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

/// A simple tool to manage encrypted secrets in YAML files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Generate the completion code for this shell
    #[arg(long, name = "SHELL")]
    pub completion: Option<clap_complete::Shell>,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, hide = true)]
    pub markdown_help: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Decrypt(DecryptArgs),
    Edit(EditArgs),
    Encrypt(EncryptArgs),
    Env(EnvArgs),
    Keygen(KeygenArgs),
    Pubkey(PubkeyArgs),
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

/// Convert private age keys to their public key
///
/// The input key and output public key are in the age format, which is compatible with the age tool.
#[derive(Args, Debug)]
pub struct PubkeyArgs {
    /// The private key files
    ///
    /// If the filename is -, the keys are read from the standard input.
    ///
    /// May be repeated.
    #[clap(name = "KEY_FILE", env = "YAGE_KEY_FILE")]
    pub key_files: Vec<PathBuf>,

    /// The private keys
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    #[clap(short, long = "key", name = "KEY", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// The output path to the public key file
    ///
    /// The public keys are written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,
}

/// Edit an encrypted YAML file
///
/// The file is decrypted with the specified keys and open in a text editor. The user can edit the file
/// and save it. The values are then encrypted with the same keys and the recipients, and saved in the
/// original file.
///
/// The YAML file may contain some unencrypted values, and some encrypted values. The encrypted values
/// are decrypted before the edition and all the values are encrypted after the edition.
///
/// Only the modified values are encrypted, the other values are left unchanged.
#[derive(Args, Debug)]
pub struct EditArgs {
    /// The editor command to use
    #[clap(short, long, env = "EDITOR")]
    pub editor: PathBuf,

    /// Decrypt with the specified key
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    #[clap(short, long = "key", name = "KEY", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key at in this file
    ///
    /// May be repeated.
    #[clap(
        short = 'K',
        long = "key-file",
        name = "KEY_FILE",
        env = "YAGE_KEY_FILE"
    )]
    pub key_files: Vec<PathBuf>,

    /// Encrypt to the specified recipients
    ///
    /// May be repeated.
    #[clap(short, long = "recipient", name = "RECIPIENT", env = "YAGE_RECIPIENT")]
    pub recipients: Vec<String>,

    /// Encrypt to recipients listed at PATH
    ///
    /// The recipients file is a text file with one recipient per line.
    ///
    /// May be repeated.
    #[clap(
        short = 'R',
        long = "recipient-file",
        name = "RECIPIENT_FILE",
        env = "YAGE_RECIPIENT_FILE"
    )]
    pub recipient_files: Vec<PathBuf>,

    /// The encrypted YAML file to edit
    #[arg()]
    pub file: PathBuf,
}

/// Encrypt the values in a YAML file
///
/// Only the values are encrypted, the keys are left in clear.
///
/// The values are encrypted with the recipients' public keys in the age format,
/// converted in base64 and surrounded by `yage[…]` markers.
///
/// This command is able to encrypt some new values in a file that already contains
/// encrypted values. The encrypted values are detected thanks to the `yage[…]` markers
/// and left unchanged.
#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// Encrypt in place
    ///
    /// The input file is overwritten with the encrypted data.
    ///
    /// The --output option is ignored if this option is used.
    #[clap(short, long)]
    pub in_place: bool,

    /// Encrypt to the specified recipients
    ///
    /// May be repeated.
    #[clap(short, long = "recipient", name = "RECIPIENT", env = "YAGE_RECIPIENT")]
    pub recipients: Vec<String>,

    /// Encrypt to recipients listed at PATH
    ///
    /// The recipients file is a text file with one recipient per line.
    ///
    /// May be repeated.
    #[clap(
        short = 'R',
        long = "recipient-file",
        name = "RECIPIENT_FILE",
        env = "YAGE_RECIPIENT_FILE"
    )]
    pub recipient_files: Vec<PathBuf>,

    /// The output path to the encrypted YAML file
    ///
    /// The encrypted YAML file is written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The YAML file to encrypt
    ///
    /// If the filename is -, the YAML file is read from the standard input.
    #[arg()]
    pub file: PathBuf,
}

/// Decrypt the values in a YAML file
#[derive(Args, Debug)]
pub struct DecryptArgs {
    /// Decrypt in place
    ///
    /// The input file is overwritten with the encrypted data.
    ///
    /// The --output option is ignored if this option is used.
    #[clap(short, long)]
    pub in_place: bool,

    /// Decrypt with the specified key
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    #[clap(short, long = "key", name = "KEY", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key in the file
    ///
    /// May be repeated.
    #[clap(
        short = 'K',
        long = "key-file",
        name = "KEY_FILE",
        env = "YAGE_KEY_FILE"
    )]
    pub key_files: Vec<PathBuf>,

    /// The output path to the decrypted YAML file
    ///
    /// The decrypted YAML file is written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The YAML file to decrypt
    #[arg()]
    pub file: PathBuf,
}

/// Execute a command with the environment from the encrypted YAML file
///
/// The YAML file must contain a map with string keys and values. The keys are the environment
/// variable names, and the values are the environment variable values.
/// Other more complex YAML structures are not supported.
#[derive(Args, Debug)]
pub struct EnvArgs {
    /// Start with an empty environment
    #[clap(short, long, default_value_t = false)]
    pub ignore_environment: bool,

    /// Decrypt with the specified key
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    #[clap(short, long = "key", name = "KEY", env = "YAGE_KEY")]
    pub keys: Vec<String>,

    /// Decrypt with the key at PATH
    ///
    /// May be repeated.
    #[clap(
        short = 'K',
        long = "key-file",
        name = "KEY_FILE",
        env = "YAGE_KEY_FILE"
    )]
    pub key_files: Vec<PathBuf>,

    /// The YAML file to decrypt
    #[arg()]
    pub file: PathBuf,

    /// The command to run
    #[arg()]
    pub command: String,

    /// The command arguments
    #[arg()]
    pub args: Vec<String>,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
