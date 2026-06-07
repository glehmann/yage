use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;
use yaml_edit::{Document, YamlNode};

use crate::error::{Result, YageError};
use crate::{EncryptionStatus, check_encrypted, check_recipients, stdin_or_file};

/// Check the encryption status of a YAML file
#[derive(Args, Debug)]
#[command(alias = "status")]
pub struct CheckArgs {
    /// The YAML files to check
    #[arg()]
    pub files: Vec<PathBuf>,
}

pub fn check(args: &CheckArgs) -> Result<i32> {
    let mut ok: bool = true;
    for file in &args.files {
        debug!("loading yaml file: {file:?}");
        // don't user read_yaml here, because we don't want it to print a warning if the
        // recipients are not consistent
        let mut s = String::new();
        stdin_or_file(file)?.read_to_string(&mut s)?;
        let doc = Document::from_str(&s).map_err(YageError::Yaml)?;
        let input_data = doc
            .as_mapping()
            .map(YamlNode::Mapping)
            .or_else(|| doc.as_sequence().map(YamlNode::Sequence))
            .or_else(|| doc.as_scalar().map(YamlNode::Scalar))
            .unwrap();
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
