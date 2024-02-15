use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;
use serde_yaml as sy;

use crate::cli::ENV_PATH_SEP;
use crate::error::{Result, YageError};
use crate::{encrypt_yaml, get_yaml_recipients, load_recipients, read_yaml, stdout_or_file};

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
    ///
    /// Multiple values may be passed in the YAGE_RECIPIENT environment variable separated by commas.
    #[clap(
        short,
        long = "recipient",
        name = "RECIPIENT",
        env = "YAGE_RECIPIENT",
        value_delimiter = ','
    )]
    pub recipients: Vec<String>,

    /// Encrypt to recipients listed at PATH
    ///
    /// The recipients file is a text file with one recipient per line.
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_RECIPIENT_FILE environment variable separated by the system path separator.
    #[clap(
        short = 'R',
        long = "recipient-file",
        name = "RECIPIENT_FILE",
        env = "YAGE_RECIPIENT_FILE",
        value_delimiter = ENV_PATH_SEP,
    )]
    pub recipient_files: Vec<PathBuf>,

    /// The output path to the encrypted YAML file
    ///
    /// The encrypted YAML file is written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The YAML files to encrypt
    ///
    /// If the filename is -, the YAML file is read from the standard input.
    #[arg()]
    pub files: Vec<PathBuf>,
}

pub fn encrypt(args: &EncryptArgs) -> Result<i32> {
    if args.in_place && args.files.contains(&PathBuf::from_str("-").unwrap()) {
        return Err(YageError::InPlaceStdin);
    }
    let recipients = load_recipients(&args.recipients, &args.recipient_files)?;
    for file in &args.files {
        let input_data = read_yaml(file)?;
        let yaml_recipients = get_yaml_recipients(&input_data)?;
        let recipients = if recipients.is_empty() {
            &yaml_recipients
        } else if yaml_recipients.is_empty() || recipients == yaml_recipients {
            &recipients
        } else {
            return Err(YageError::InvalidRecipients);
        };
        let output_data = encrypt_yaml(&input_data, recipients)?;
        let output = stdout_or_file(if args.in_place { file } else { &args.output })?;
        sy::to_writer(output, &output_data)?;
    }
    Ok(0)
}
