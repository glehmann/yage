#[macro_use]
extern crate log;

pub mod cli;
pub mod error;

pub mod cmd {
    mod check;
    mod decrypt;
    mod edit;
    mod encrypt;
    mod env;
    mod keygen;
    mod pubkey;
    pub use check::check;
    pub use decrypt::decrypt;
    pub use edit::edit;
    pub use encrypt::encrypt;
    pub use env::env;
    pub use keygen::keygen;
    pub use pubkey::pubkey;
}

use std::fs::{File, OpenOptions};
use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use age::x25519;
use base64::prelude::*;
use serde_yaml as sy;
use strum::{Display, EnumIs, EnumIter, EnumString};
use substring::Substring;

use crate::error::{IOResultExt, Result, YageError};

pub fn stdout_or_file(path: &Path) -> Result<Box<dyn Write>> {
    Ok(if path == Path::new("-") {
        Box::new(stdout())
    } else {
        Box::new(File::create(path).path_ctx(path)?)
    })
}

pub fn stdout_or_private_file(path: &Path) -> Result<Box<dyn Write>> {
    Ok(if path == Path::new("-") {
        Box::new(stdout())
    } else {
        Box::new(create_private_file(path)?)
    })
}

pub fn create_private_file(path: &Path) -> Result<File> {
    let mut file_opts = OpenOptions::new();
    file_opts.write(true).create_new(true);
    #[cfg(unix)]
    file_opts.mode(0o600);
    file_opts.open(path).path_ctx(path)
}

pub fn stdin_or_file(path: &Path) -> Result<BufReader<Box<dyn Read>>> {
    Ok(if path == Path::new("-") {
        BufReader::new(Box::new(stdin()))
    } else {
        BufReader::new(Box::new(File::open(path).path_ctx(path)?))
    })
}

pub fn stdin_or_private_file(path: &Path) -> Result<BufReader<Box<dyn Read>>> {
    Ok(if path == Path::new("-") {
        BufReader::new(Box::new(stdin()))
    } else {
        let br: BufReader<Box<dyn Read>> =
            BufReader::new(Box::new(File::open(path).path_ctx(path)?));
        if let Err(e) = fs_mistrust::Mistrust::new()
            .verifier()
            .require_file()
            .check(path)
        {
            warn!("file {path:?} is not private: {e}");
        }
        br
    })
}

pub fn is_yage_encoded(s: &str) -> bool {
    s.starts_with("yage[") && s.ends_with(']')
}

pub fn decrypt_yaml(value: &sy::Value, identities: &[x25519::Identity]) -> Result<sy::Value> {
    match value {
        sy::Value::Mapping(mapping) => {
            let mut output = sy::Mapping::new();
            for (key, value) in mapping {
                let key = key.clone();
                let value = decrypt_yaml(value, identities)?;
                output.insert(key, value);
            }
            Ok(sy::Value::Mapping(output))
        }
        sy::Value::Sequence(sequence) => {
            let mut output = Vec::new();
            for value in sequence {
                let value = decrypt_yaml(value, identities)?;
                output.push(value);
            }
            Ok(sy::Value::Sequence(output))
        }
        sy::Value::String(encrypted) => {
            let decrypted = decrypt_value(encrypted, identities)?;
            Ok(decrypted)
        }
        _ => Ok(value.clone()),
    }
}

pub fn decrypt_value(s: &str, identities: &[x25519::Identity]) -> Result<sy::Value> {
    if is_yage_encoded(s) {
        // remove the yage[…] prefix and suffix
        let encoded = s.substring(5, s.len() - 1);
        let encrypted = BASE64_STANDARD.decode(encoded)?;
        let decryptor = match age::Decryptor::new(&encrypted[..])? {
            age::Decryptor::Recipients(d) => Ok(d),
            _ => Err(YageError::PassphraseUnsupported),
        }?;
        let mut decrypted = vec![];
        let mut reader = decryptor.decrypt(identities.iter().map(|i| i as &dyn age::Identity))?;
        reader.read_to_end(&mut decrypted)?;
        let value: sy::Value = sy::from_slice(&decrypted)?;
        Ok(value)
    } else {
        Ok(sy::Value::String(s.to_owned()))
    }
}

pub fn load_identities(keys: &[String], key_files: &[PathBuf]) -> Result<Vec<x25519::Identity>> {
    let mut identities: Vec<x25519::Identity> = Vec::new();
    for key in keys.iter() {
        debug!("loading key: {key}");
        let key = x25519::Identity::from_str(key)
            .map_err(|e| YageError::KeyParse { message: e.into() })?;
        identities.push(key);
    }
    for key_file in key_files.iter() {
        debug!("loading key file: {key_file:?}");
        let input = stdin_or_private_file(key_file)?;
        let keys = age::IdentityFile::from_buffer(input).path_ctx(key_file)?;
        for key in keys.into_identities() {
            let age::IdentityFileEntry::Native(key) = key;
            identities.push(key);
        }
    }
    Ok(identities)
}

pub fn encrypt_yaml(value: &sy::Value, recipients: &[x25519::Recipient]) -> Result<sy::Value> {
    match value {
        sy::Value::Mapping(mapping) => {
            let mut output = sy::Mapping::new();
            for (key, value) in mapping {
                let key = key.clone();
                let value = encrypt_yaml(value, recipients)?;
                output.insert(key, value);
            }
            Ok(sy::Value::Mapping(output))
        }
        sy::Value::Sequence(sequence) => {
            let mut output = Vec::new();
            for value in sequence {
                let value = encrypt_yaml(value, recipients)?;
                output.push(value);
            }
            Ok(sy::Value::Sequence(output))
        }
        sy::Value::String(s) => {
            let output = if is_yage_encoded(s) {
                // keep the already encrypted value
                s.to_owned()
            } else {
                encrypt_value(value, recipients)?
            };
            Ok(sy::Value::String(output))
        }
        sy::Value::Number(_) => {
            let output = encrypt_value(value, recipients)?;
            Ok(sy::Value::String(output))
        }
        _ => Ok(value.clone()),
    }
}

pub fn encrypt_value(value: &sy::Value, recipients: &[x25519::Recipient]) -> Result<String> {
    type Recipients = Vec<Box<dyn age::Recipient + Send + 'static>>;
    let data = sy::to_string(value)?;
    let recipients = recipients
        .iter()
        .map(|r| Box::new(r.clone()) as Box<dyn age::Recipient + Send + 'static>)
        .collect::<Recipients>();
    let mut encrypted = vec![];
    let encryptor = age::Encryptor::with_recipients(recipients).ok_or(YageError::NoRecipients)?;
    // let mut armored = ArmoredWriter::wrap_output(&mut encrypted, Format::AsciiArmor)?;
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(data.as_bytes())?;
    writer.finish()?;
    let encoded = BASE64_STANDARD.encode(&encrypted);
    Ok(format!("yage[{encoded}]"))
}

pub fn load_recipients(
    recipients: &[String],
    recipients_paths: &[PathBuf],
) -> Result<Vec<x25519::Recipient>> {
    let mut res: Vec<x25519::Recipient> = Vec::new();
    // read the recipient from the command line
    for recipient in recipients.iter() {
        debug!("loading recipient: {recipient}");
        let recipient =
            x25519::Recipient::from_str(recipient).map_err(|e| YageError::RecipientParse {
                recipient: recipient.to_owned(),
                message: e.into(),
            })?;
        res.push(recipient);
    }
    // read the recipient from the files
    for path in recipients_paths.iter() {
        debug!("loading recipient file: {path:?}");
        let input = stdin_or_file(path)?;
        for recipient in input.lines() {
            let recipient = recipient.path_ctx(path)?;
            let recipient =
                x25519::Recipient::from_str(&recipient).map_err(|e| YageError::RecipientParse {
                    recipient: recipient.to_owned(),
                    message: e.into(),
                })?;
            res.push(recipient);
        }
    }
    Ok(res)
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString, EnumIs, EnumIter)]
pub enum EncryptionStatus {
    Encrypted,
    NotEncrypted,
    Mixed,
    NoValue,
}

pub fn check_encrypted(value: &sy::Value) -> EncryptionStatus {
    match value {
        sy::Value::Mapping(mapping) => check_encrypted_iter(mapping.iter().map(|(_, v)| v)),
        sy::Value::Sequence(sequence) => check_encrypted_iter(sequence.iter()),
        sy::Value::String(s) => {
            if is_yage_encoded(s) {
                EncryptionStatus::Encrypted
            } else {
                EncryptionStatus::NotEncrypted
            }
        }
        sy::Value::Null => EncryptionStatus::NoValue,
        _ => EncryptionStatus::NotEncrypted,
    }
}

fn check_encrypted_iter<'a>(iter: impl Iterator<Item = &'a sy::Value>) -> EncryptionStatus {
    let mut status = EncryptionStatus::NoValue;
    for value in iter {
        match check_encrypted(value) {
            EncryptionStatus::Encrypted => {
                status = match status {
                    EncryptionStatus::Encrypted => EncryptionStatus::Encrypted,
                    EncryptionStatus::NotEncrypted => EncryptionStatus::Mixed,
                    EncryptionStatus::Mixed => EncryptionStatus::Mixed,
                    EncryptionStatus::NoValue => EncryptionStatus::Encrypted,
                }
            }
            EncryptionStatus::NotEncrypted => {
                status = match status {
                    EncryptionStatus::Encrypted => EncryptionStatus::Mixed,
                    EncryptionStatus::NotEncrypted => EncryptionStatus::NotEncrypted,
                    EncryptionStatus::Mixed => EncryptionStatus::Mixed,
                    EncryptionStatus::NoValue => EncryptionStatus::NotEncrypted,
                }
            }
            EncryptionStatus::Mixed => {
                status = EncryptionStatus::Mixed;
            }
            EncryptionStatus::NoValue => (),
        }
    }
    status
}
