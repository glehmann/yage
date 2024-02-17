use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;

use crate::cli::ENV_PATH_SEP;
use crate::error::{Result, YageError};
use crate::{
    decrypt_yaml, encrypt_yaml, get_yaml_recipients, load_identities, load_recipients, read_yaml,
    write_yaml,
};

/// Re-encrypt the values in a YAML file
///
/// This command is similar to the encrypt command, but it first decrypts the values in the input
/// file and re-encrypts them with the specified recipients.
///
/// It is especially useful in several cases:
/// - to change the recipients of a file, for example when a recipient's key is compromised,
///   or when a recipient should be added or removed from the file.
/// - to fix a recipient inconsistency in the values of a file, for example when the file was
///   merged from different sources with different recipients.
#[derive(Args, Debug)]
#[command(alias = "recrypt")]
pub struct ReEncryptArgs {
    /// Re-encrypt in place
    ///
    /// The input file is overwritten with the encrypted data.
    ///
    /// The --output option is ignored if this option is used.
    #[clap(short, long)]
    pub in_place: bool,

    /// Keep the recipients of the input file
    ///
    /// More recipients may be added with the --recipient and --recipient-file options,
    /// and removed with the --remove-recipient and --remove-recipient-file options.
    #[clap(short = 'e', long)]
    pub keep_recipients: bool,

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

    /// Encrypt to the specified recipients
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_RECIPIENT environment variable separated by commas.
    #[clap(
        short,
        long = "recipient",
        value_name = "RECIPIENT",
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
        value_name = "FILE",
        env = "YAGE_RECIPIENT_FILE",
        value_delimiter = ENV_PATH_SEP,
    )]
    pub recipient_files: Vec<PathBuf>,

    /// Remove the recipient from the list of recipients
    ///
    /// The removal in the recipient list is always done after processing the --keep-recipients,
    /// --recipient and --recipient-file options.
    ///
    /// May be repeated.
    #[clap(short = 'd', long = "remove-recipient", value_name = "RECIPIENT")]
    pub remove_recipients: Vec<String>,

    /// Remove the recipients in the file from the list of recipients
    ///
    /// The removal in the recipient list is always done after processing the --keep-recipients,
    /// --recipient and --recipient-file options.
    ///
    /// May be repeated.
    #[clap(short = 'D', long = "remove-recipient-file", value_name = "FILE")]
    pub remove_recipient_files: Vec<PathBuf>,

    /// The output path to the encrypted YAML file
    ///
    /// The encrypted YAML file is written to the standard output by default.
    #[clap(short, long, default_value = "-", value_name = "FILE")]
    pub output: PathBuf,

    /// The YAML files to encrypt
    ///
    /// If the filename is -, the YAML file is read from the standard input.
    #[arg()]
    pub files: Vec<PathBuf>,
}

pub fn re_encrypt(args: &ReEncryptArgs) -> Result<i32> {
    if args.in_place && args.files.contains(&PathBuf::from_str("-").unwrap()) {
        return Err(YageError::InPlaceStdin);
    }
    if !args.in_place && args.files.len() != 1 {
        return Err(YageError::InvalidNumberOfInputFiles);
    }
    let identities = load_identities(&args.keys, &args.key_files)?;
    let arg_recipients = load_recipients(&args.recipients, &args.recipient_files)?;
    let remove_recipients = load_recipients(&args.remove_recipients, &args.remove_recipient_files)?;
    for file in &args.files {
        let input_data = read_yaml(file)?;
        let decrypted_data = decrypt_yaml(&input_data, &identities)?;
        let yaml_recipients =
            if args.keep_recipients { get_yaml_recipients(&input_data)? } else { vec![] };
        let mut recipients = [arg_recipients.clone(), yaml_recipients].concat();
        recipients.sort_by_cached_key(|r| r.to_string());
        recipients.dedup();
        recipients.retain(|r| !remove_recipients.contains(r));
        debug!("{file:?} recipients: {recipients:?}");
        let output_data = encrypt_yaml(&decrypted_data, &recipients)?;
        write_yaml(if args.in_place { file } else { &args.output }, &output_data)?;
    }
    Ok(0)
}
