use serde_yaml as sy;

use crate::cli::StatusArgs;
use crate::error::Result;
use crate::{check_encrypted, stdin_or_file, EncryptionStatus};

pub fn status(args: &StatusArgs) -> Result<i32> {
    let mut is_encrypted: bool = true;
    for file in &args.files {
        debug!("loading yaml file: {file:?}");
        let input_data: sy::Value = sy::from_reader(stdin_or_file(file)?)?;
        match check_encrypted(&input_data) {
            EncryptionStatus::Encrypted => (),
            EncryptionStatus::Mixed => {
                error! {"{file:?} is partially encrypted"};
                is_encrypted = false;
            }
            EncryptionStatus::NotEncrypted => {
                error! {"{file:?} is not encrypted"};
                is_encrypted = false;
            }
        }
    }
    Ok(if is_encrypted { 0 } else { 1 })
}
