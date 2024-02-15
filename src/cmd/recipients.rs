use std::collections::HashSet;
use std::path::PathBuf;

use age::x25519;
use clap::Args;
use serde_yaml as sy;

use crate::error::{IOResultExt, Result};
use crate::{get_yaml_recipients, stdin_or_file, stdout_or_file};

/// List the recipients of the encrypted data
#[derive(Args, Debug)]
pub struct RecipientsArgs {
    /// Only show the recipients' public keys
    ///
    /// The file names are not shown and the recipients from the different files are merged,
    /// deduplicated and sorted.
    #[clap(short = 'r', long, default_value_t = false)]
    pub only_recipients: bool,

    /// The output path
    ///
    /// The recipients are written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The encrypted YAML files
    #[arg()]
    pub files: Vec<PathBuf>,
}

pub fn recipients(args: &RecipientsArgs) -> Result<i32> {
    let mut output = stdout_or_file(&args.output)?;
    if args.only_recipients {
        let mut recipients: HashSet<x25519::Recipient> = HashSet::new();
        for file in &args.files {
            let input_data = sy::from_reader(stdin_or_file(file)?)?;
            let file_recipients = get_yaml_recipients(&input_data)?;
            recipients.extend(file_recipients);
        }
        let mut recipients: Vec<_> = recipients.iter().map(|r| r.to_string()).collect();
        recipients.sort();
        for recipient in &recipients {
            writeln!(output, "{}", recipient).path_ctx(&args.output)?;
        }
    } else {
        for file in &args.files {
            let input_data = sy::from_reader(stdin_or_file(file)?)?;
            let recipients = get_yaml_recipients(&input_data)?;
            let file = file.to_string_lossy();
            for recipient in recipients {
                writeln!(output, "{file}: {recipient}").path_ctx(&args.output)?;
            }
        }
    }
    Ok(0)
}