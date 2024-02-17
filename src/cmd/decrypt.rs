use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;

use crate::cli::ENV_PATH_SEP;
use crate::error::{Result, YageError};
use crate::{decrypt_yaml, load_identities, read_yaml, write_yaml};

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
    ///
    /// Multiple values may be passed in the YAGE_KEY environment variable separated by commas.
    #[clap(short, long = "key", value_name = "KEY", env = "YAGE_KEY", value_delimiter = ',')]
    pub keys: Vec<String>,

    /// Decrypt with the key in the file
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_KEY_FILE environment variable separated by the system path separator.
    #[clap(
        short = 'K',
        long = "key-file",
        value_name = "FILE",
        env = "YAGE_KEY_FILE",
        value_delimiter = ENV_PATH_SEP,
    )]
    pub key_files: Vec<PathBuf>,

    /// The output path to the decrypted YAML file
    ///
    /// The decrypted YAML file is written to the standard output by default.
    #[clap(short, long, default_value = "-", value_name = "FILE")]
    pub output: PathBuf,

    /// The YAML files to decrypt
    #[arg()]
    pub files: Vec<PathBuf>,
}

pub fn decrypt(args: &DecryptArgs) -> Result<i32> {
    if args.in_place && args.files.contains(&PathBuf::from_str("-").unwrap()) {
        return Err(YageError::InPlaceStdin);
    }
    if !args.in_place && args.files.len() != 1 {
        return Err(YageError::InvalidNumberOfInputFiles);
    }
    let identities = load_identities(&args.keys, &args.key_files)?;
    for file in &args.files {
        let input_data = read_yaml(file)?;
        let output_data = decrypt_yaml(&input_data, &identities)?;
        write_yaml(if args.in_place { file } else { &args.output }, &output_data)?;
    }
    Ok(0)
}
