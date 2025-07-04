use std::collections::HashSet;
use std::path::PathBuf;

use age::x25519;
use clap::Args;

use crate::error::{IOResultExt, Result};
use crate::{get_yaml_recipients, read_yaml, stdout_or_file};

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
    #[clap(short, long, default_value = "-", value_name = "FILE")]
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
            recipients.extend(get_yaml_recipients(&read_yaml(file)?)?);
        }
        let mut recipients: Vec<_> = recipients.iter().map(|r| r.to_string()).collect();
        recipients.sort();
        for recipient in &recipients {
            writeln!(output, "{recipient}").path_ctx(&args.output)?;
        }
    } else {
        for file in &args.files {
            let recipients = get_yaml_recipients(&read_yaml(file)?)?;
            let file = file.to_string_lossy();
            for recipient in recipients {
                writeln!(output, "{file}: {recipient}").path_ctx(&args.output)?;
            }
        }
    }
    Ok(0)
}
