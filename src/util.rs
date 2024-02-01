use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use age::x25519;
use base64::prelude::*;
use substring::Substring;

use crate::error::{AppError, IOResultExt, Result};

pub fn stdout_or_file(path: &Path) -> Result<Box<dyn Write>> {
    Ok(if path == Path::new("-") {
        Box::new(stdout())
    } else {
        Box::new(File::create(path).path_ctx(path)?)
    })
}

pub fn stdin_or_file(path: &Path) -> Result<BufReader<Box<dyn Read>>> {
    Ok(if path == Path::new("-") {
        BufReader::new(Box::new(stdin()))
    } else {
        BufReader::new(Box::new(File::open(path).path_ctx(path)?))
    })
}

pub fn is_yage_encoded(s: &str) -> bool {
    s.starts_with("yage[") && s.ends_with("]")
}

pub fn decrypt_yaml(
    value: &serde_yaml::Value,
    identities: &[x25519::Identity],
) -> Result<serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            let mut output = serde_yaml::Mapping::new();
            for (key, value) in mapping {
                let key = key.clone();
                let value = decrypt_yaml(value, identities)?;
                output.insert(key, value);
            }
            Ok(serde_yaml::Value::Mapping(output))
        }
        serde_yaml::Value::Sequence(sequence) => {
            let mut output = Vec::new();
            for value in sequence {
                let value = decrypt_yaml(value, identities)?;
                output.push(value);
            }
            Ok(serde_yaml::Value::Sequence(output))
        }
        serde_yaml::Value::String(encrypted) => {
            let decrypted = decrypt_value(encrypted, identities)?;
            Ok(decrypted)
        }
        _ => Ok(value.clone()),
    }
}

pub fn decrypt_value(s: &str, identities: &[x25519::Identity]) -> Result<serde_yaml::Value> {
    if is_yage_encoded(s) {
        // remove the yage[…] prefix and suffix
        let encoded = s.substring(5, s.len() - 1);
        let encrypted = BASE64_STANDARD.decode(encoded)?;
        let decryptor = match age::Decryptor::new(&encrypted[..])? {
            age::Decryptor::Recipients(d) => Ok(d),
            _ => Err(AppError::PassphraseUnsupportedError),
        }?;
        let mut decrypted = vec![];
        let mut reader = decryptor.decrypt(identities.iter().map(|i| i as &dyn age::Identity))?;
        reader.read_to_end(&mut decrypted)?;
        let value: serde_yaml::Value = serde_yaml::from_slice(&decrypted)?;
        Ok(value)
    } else {
        Ok(serde_yaml::Value::String(s.to_owned()))
    }
}

pub fn load_identities(keys: &[String], key_files: &[PathBuf]) -> Result<Vec<x25519::Identity>> {
    let mut identities: Vec<x25519::Identity> = Vec::new();
    for key in keys.iter() {
        debug!("loading key: {key}");
        let key = x25519::Identity::from_str(key)
            .map_err(|e| AppError::KeyParseError { message: e.into() })?;
        identities.push(key);
    }
    for key_file in key_files.iter() {
        debug!("loading key file: {key_file:?}");
        let input = stdin_or_file(key_file)?;
        let keys = age::IdentityFile::from_buffer(input).path_ctx(key_file)?;
        for key in keys.into_identities() {
            let age::IdentityFileEntry::Native(key) = key;
            identities.push(key);
        }
    }
    Ok(identities)
}

pub fn encrypt_yaml(
    value: &serde_yaml::Value,
    recipients: &[x25519::Recipient],
) -> Result<serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            let mut output = serde_yaml::Mapping::new();
            for (key, value) in mapping {
                let key = key.clone();
                let value = encrypt_yaml(value, recipients)?;
                output.insert(key, value);
            }
            Ok(serde_yaml::Value::Mapping(output))
        }
        serde_yaml::Value::Sequence(sequence) => {
            let mut output = Vec::new();
            for value in sequence {
                let value = encrypt_yaml(value, recipients)?;
                output.push(value);
            }
            Ok(serde_yaml::Value::Sequence(output))
        }
        serde_yaml::Value::String(s) => {
            let output = if is_yage_encoded(s) {
                // keep the already encrypted value
                s.to_owned()
            } else {
                encrypt_value(value, recipients)?
            };
            Ok(serde_yaml::Value::String(output))
        }
        serde_yaml::Value::Number(_) => {
            let output = encrypt_value(value, recipients)?;
            Ok(serde_yaml::Value::String(output))
        }
        _ => Ok(value.clone()),
    }
}

pub fn encrypt_value(
    value: &serde_yaml::Value,
    recipients: &[x25519::Recipient],
) -> Result<String> {
    type Recipients = Vec<Box<dyn age::Recipient + Send + 'static>>;
    let data = serde_yaml::to_string(value)?;
    let recipients = recipients
        .iter()
        .map(|r| Box::new(r.clone()) as Box<dyn age::Recipient + Send + 'static>)
        .collect::<Recipients>();
    let mut encrypted = vec![];
    let encryptor =
        age::Encryptor::with_recipients(recipients).ok_or(AppError::NoRecipientsError)?;
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
            x25519::Recipient::from_str(recipient).map_err(|e| AppError::RecipientParseError {
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
            let recipient = x25519::Recipient::from_str(&recipient).map_err(|e| {
                AppError::RecipientParseError {
                    recipient: recipient.to_owned(),
                    message: e.into(),
                }
            })?;
            res.push(recipient);
        }
    }
    Ok(res)
}
