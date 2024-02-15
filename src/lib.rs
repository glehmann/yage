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
    mod recipients;
    pub use check::*;
    pub use decrypt::*;
    pub use edit::*;
    pub use encrypt::*;
    pub use env::*;
    pub use keygen::*;
    pub use pubkey::*;
    pub use recipients::*;
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
    match YageEncodedValue::from_str(s) {
        Ok(yev) => {
            let encrypted = BASE64_STANDARD.decode(yev.data)?;
            let decryptor = match age::Decryptor::new(&encrypted[..])? {
                age::Decryptor::Recipients(d) => Ok(d),
                _ => Err(YageError::PassphraseUnsupported),
            }?;
            let mut decrypted = vec![];
            let mut reader =
                decryptor.decrypt(identities.iter().map(|i| i as &dyn age::Identity))?;
            reader.read_to_end(&mut decrypted)?;
            let value: sy::Value = sy::from_slice(&decrypted)?;
            Ok(value)
        }
        Err(_) => Ok(sy::Value::String(s.to_owned())),
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
            let output = if YageEncodedValue::from_str(s).is_ok() {
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
    let recipients_dyn = recipients
        .iter()
        .map(|r| Box::new(r.clone()) as Box<dyn age::Recipient + Send + 'static>)
        .collect::<Recipients>();
    let mut encrypted = vec![];
    let encryptor =
        age::Encryptor::with_recipients(recipients_dyn).ok_or(YageError::NoRecipients)?;
    // let mut armored = ArmoredWriter::wrap_output(&mut encrypted, Format::AsciiArmor)?;
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(data.as_bytes())?;
    writer.finish()?;
    let mut recipients: Vec<_> = recipients.iter().map(|r| r.to_string()).collect();
    recipients.sort();
    recipients.dedup();
    let yev = YageEncodedValue {
        data: BASE64_STANDARD.encode(&encrypted),
        recipients,
    };
    Ok(yev.to_string())
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
    res.sort_by_cached_key(|r| r.to_string());
    res.dedup();
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
            if YageEncodedValue::from_str(s).is_ok() {
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

pub fn flatten_yage_encrypted_values(value: &sy::Value) -> Vec<YageEncodedValue> {
    match value {
        sy::Value::Mapping(mapping) => mapping
            .iter()
            .flat_map(|(_, v)| flatten_yage_encrypted_values(v))
            .collect(),
        sy::Value::Sequence(sequence) => sequence
            .iter()
            .flat_map(flatten_yage_encrypted_values)
            .collect(),
        sy::Value::String(s) => match YageEncodedValue::from_str(s) {
            Ok(yev) => vec![yev],
            Err(_) => vec![],
        },
        _ => vec![],
    }
}

pub fn check_recipients(value: &sy::Value) -> bool {
    flatten_yage_encrypted_values(value)
        .iter()
        .filter(|v| !v.recipients.is_empty())
        .map(|v| &v.recipients)
        .collect::<Vec<_>>()
        .windows(2)
        .all(|w| w[0] == w[1])
}

#[derive(Debug, Clone)]
pub struct YageEncodedValue {
    pub data: String,
    pub recipients: Vec<String>,
}

impl FromStr for YageEncodedValue {
    type Err = YageError;

    fn from_str(s: &str) -> Result<Self> {
        if !s.starts_with("yage[") || !s.ends_with(']') {
            return Err(YageError::InvalidValueEncoding);
        }
        // remove the yage[…] prefix and suffix
        let payload = s.substring(5, s.len() - 1);
        let components: Vec<_> = payload.split('|').collect();
        if components.len() != 2 {
            return Err(YageError::InvalidValueEncoding);
        }
        let data = components[0].to_owned();
        if !components[1].starts_with("r:") {
            return Err(YageError::InvalidValueEncoding);
        }
        let recipients = components[1].substring(2, components[1].len());
        let recipients: Vec<String> = recipients.split(',').map(|r| r.to_owned()).collect();
        Ok(YageEncodedValue { data, recipients })
    }
}

impl std::fmt::Display for YageEncodedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let recipients = self.recipients.join(",");
        write!(f, "yage[{}|r:{}]", self.data, recipients)
    }
}

pub fn get_yaml_recipients(value: &sy::Value) -> Result<Vec<x25519::Recipient>> {
    let yevs = flatten_yage_encrypted_values(value);
    let mut recipients: Vec<_> = yevs.iter().flat_map(|yev| &yev.recipients).collect();
    recipients.sort();
    recipients.dedup();
    let mut output: Vec<x25519::Recipient> = Vec::with_capacity(recipients.len());
    for s in recipients {
        let r = x25519::Recipient::from_str(s).map_err(|msg| YageError::RecipientParse {
            recipient: s.to_owned(),
            message: msg.to_owned(),
        })?;
        output.push(r);
    }
    Ok(output)
}
