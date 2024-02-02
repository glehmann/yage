use serde_yaml as sy;

use crate::cli::EncryptArgs;
use crate::error::Result;
use crate::util::{encrypt_yaml, load_recipients, stdin_or_file, stdout_or_file};

pub fn encrypt(args: &EncryptArgs) -> Result<()> {
    let recipients = load_recipients(&args.recipients, &args.recipients_files)?;
    debug!("loading yaml file: {:?}", args.file);
    let input_data: sy::Value = sy::from_reader(stdin_or_file(&args.file)?)?;
    let output_data = encrypt_yaml(&input_data, &recipients)?;
    let output = stdout_or_file(if args.in_place {
        &args.file
    } else {
        &args.output
    })?;
    sy::to_writer(output, &output_data)?;
    Ok(())
}
