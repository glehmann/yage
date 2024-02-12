use std::path::PathBuf;
use std::str::FromStr;

use serde_yaml as sy;

use crate::cli::EncryptArgs;
use crate::error::{Result, YageError};
use crate::{encrypt_yaml, get_yaml_recipients, load_recipients, stdin_or_file, stdout_or_file};

pub fn encrypt(args: &EncryptArgs) -> Result<i32> {
    if args.in_place && args.files.contains(&PathBuf::from_str("-").unwrap()) {
        return Err(YageError::InPlaceStdin);
    }
    let recipients = load_recipients(&args.recipients, &args.recipient_files)?;
    for file in &args.files {
        debug!("loading yaml file: {file:?}");
        let input_data: sy::Value = sy::from_reader(stdin_or_file(file)?)?;
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
