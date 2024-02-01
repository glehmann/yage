use std::io::Read;
use std::str::FromStr;

use age::x25519;
use base64::prelude::*;
use substring::Substring;

use crate::cli::DecryptArgs;
use crate::error::{AppError, IOResultExt, Result};
use crate::util::{is_yage_encoded, stdin_or_file, stdout_or_file};

pub fn decrypt(args: &DecryptArgs) -> Result<()> {
    let mut identities: Vec<x25519::Identity> = Vec::new();
    for key in args.keys.iter() {
        debug!("loading key: {key}");
        let key = x25519::Identity::from_str(key)
            .map_err(|e| AppError::KeyParseError { message: e.into() })?;
        identities.push(key);
    }
    for key_file in args.key_files.iter() {
        debug!("loading key file: {key_file:?}");
        let input = stdin_or_file(key_file)?;
        let keys = age::IdentityFile::from_buffer(input).path_ctx(key_file)?;
        for key in keys.into_identities() {
            let age::IdentityFileEntry::Native(key) = key;
            identities.push(key);
        }
    }
    debug!("loading yaml file: {:?}", args.file);
    let input_data: serde_yaml::Value = serde_yaml::from_reader(stdin_or_file(&args.file)?)?;
    let output_data = decrypt_yaml(&input_data, &identities)?;
    let output = stdout_or_file(if args.inplace {
        &args.file
    } else {
        &args.output
    })?;
    serde_yaml::to_writer(output, &output_data)?;
    Ok(())
}

fn decrypt_yaml(
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

fn decrypt_value(s: &str, identities: &[x25519::Identity]) -> Result<serde_yaml::Value> {
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
