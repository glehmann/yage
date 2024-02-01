use std::io::{BufRead, Write};
use std::str::FromStr;

// use age::armor::{ArmoredWriter, Format};
use age::x25519;
use age::Encryptor;
use base64::prelude::*;

use crate::cli::EncryptArgs;
use crate::error::{AppError, IOResultExt, Result};
use crate::util::{is_yage_encoded, stdin_or_file, stdout_or_file};

type Recipients = Vec<Box<dyn age::Recipient + Send + 'static>>;

pub fn encrypt(args: &EncryptArgs) -> Result<()> {
    let mut recipients: Vec<x25519::Recipient> = Vec::new();
    // read the recipient from the command line
    for recipient in args.recipients.iter() {
        debug!("loading recipient: {recipient}");
        let recipient =
            x25519::Recipient::from_str(recipient).map_err(|e| AppError::RecipientParseError {
                recipient: recipient.to_owned(),
                message: e.into(),
            })?;
        recipients.push(recipient);
    }
    // read the recipient from the files
    for path in args.recipients_paths.iter() {
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
            recipients.push(recipient);
        }
    }
    debug!("loading yaml file: {:?}", args.file);
    let input_data: serde_yaml::Value = serde_yaml::from_reader(stdin_or_file(&args.file)?)?;
    let output_data = encrypt_yaml(&input_data, &recipients)?;
    let output = stdout_or_file(if args.inplace {
        &args.file
    } else {
        &args.output
    })?;
    serde_yaml::to_writer(output, &output_data)?;
    Ok(())
}

fn encrypt_yaml(
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

fn encrypt_value(value: &serde_yaml::Value, recipients: &[x25519::Recipient]) -> Result<String> {
    let data = serde_yaml::to_string(value)?;
    let recipients = recipients
        .iter()
        .map(|r| Box::new(r.clone()) as Box<dyn age::Recipient + Send + 'static>)
        .collect::<Recipients>();
    let mut encrypted = vec![];
    let encryptor = Encryptor::with_recipients(recipients).ok_or(AppError::NoRecipientsError)?;
    // let mut armored = ArmoredWriter::wrap_output(&mut encrypted, Format::AsciiArmor)?;
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(data.as_bytes())?;
    writer.finish()?;
    let encoded = BASE64_STANDARD.encode(&encrypted);
    Ok(format!("yage[{encoded}]"))
}
