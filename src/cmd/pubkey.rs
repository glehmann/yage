use std::path::PathBuf;

use clap::Args;

use crate::cli::ENV_PATH_SEP;
use crate::error::{IOResultExt, Result};
use crate::{load_identities, stdout_or_file};

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
    ///
    /// Multiple values may be passed in the YAGE_KEY_FILE environment variable separated by the system path separator.
    #[clap(name = "KEY_FILE", env = "YAGE_KEY_FILE", value_delimiter = ENV_PATH_SEP)]
    pub key_files: Vec<PathBuf>,

    /// The private keys
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_KEY environment variable separated by commas.
    #[clap(short, long = "key", name = "KEY", env = "YAGE_KEY", value_delimiter = ',')]
    pub keys: Vec<String>,

    /// The output path to the public key file
    ///
    /// The public keys are written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,
}

pub fn pubkey(args: &PubkeyArgs) -> Result<i32> {
    let keys = load_identities(&args.keys, &args.key_files)?;
    let mut output = stdout_or_file(&args.output)?;
    for key in keys {
        writeln!(output, "{}", key.to_public()).path_ctx(&args.output)?;
    }
    Ok(0)
}
