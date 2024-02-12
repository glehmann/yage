use serde_yaml as sy;

use crate::cli::CheckArgs;
use crate::error::Result;
use crate::{check_encrypted, check_recipients, stdin_or_file, EncryptionStatus};

pub fn check(args: &CheckArgs) -> Result<i32> {
    let mut ok: bool = true;
    for file in &args.files {
        debug!("loading yaml file: {file:?}");
        let input_data: sy::Value = sy::from_reader(stdin_or_file(file)?)?;
        if !check_recipients(&input_data) {
            error! {"{file:?}: inconsistent recipients"};
            ok = false;
        }
        match check_encrypted(&input_data) {
            EncryptionStatus::Encrypted | EncryptionStatus::NoValue => (),
            EncryptionStatus::Mixed => {
                error! {"{file:?}: partially encrypted"};
                ok = false;
            }
            EncryptionStatus::NotEncrypted => {
                error! {"{file:?}: not encrypted"};
                ok = false;
            }
        }
    }
    Ok(if ok { 0 } else { 1 })
}
