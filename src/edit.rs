use std::fs::File;

use tempfile::tempdir;
use treediff::tools::{ChangeType, Recorder};
use treediff::value::Key;
use treediff::Mutable;

use crate::cli::EditArgs;
use crate::error::{AppError, IOResultExt, Result};
use crate::util::{decrypt_yaml, encrypt_yaml, load_identities, load_recipients};

pub fn edit(args: &EditArgs) -> Result<()> {
    let identities = load_identities(&args.keys, &args.key_files)?;
    let mut recipients = load_recipients(&args.recipients, &args.recipients_paths)?;
    // add the identities to the recipients, so that the user can edit the file
    for identity in &identities {
        recipients.push(identity.to_public());
    }
    debug!("loading yaml file: {:?}", args.file);
    let input_data: serde_yaml::Value =
        serde_yaml::from_reader(File::open(&args.file).path_ctx(&args.file)?)?;
    let previous_data = decrypt_yaml(&input_data, &identities)?;
    // save the decrypted data in an editable temporary file. The file has the same name as the
    // original file, but in a temporary directory. This way the user knows which file he is
    // editing if its editor shows the file name.
    let dir = tempdir()?;
    let filename = args
        .file
        .file_name()
        .ok_or(AppError::InvalidFileNameError {
            path: args.file.clone(),
        })?;
    let temp_file = dir.path().join(filename);
    {
        let output = File::create(&temp_file).path_ctx(&temp_file)?;
        serde_yaml::to_writer(output, &previous_data)?;
    }
    // open the editor
    let status = std::process::Command::new(&args.editor)
        .arg(&temp_file)
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(AppError::EditorError);
    }
    // load the data edited by the user
    let edited_data: serde_yaml::Value = serde_yaml::from_reader(File::open(&temp_file)?)?;

    // find what has not changed, and keep the encrypted data unchanged. That data is encrypted
    // with a nonce that make it appear different every time it is encrypted, so we avoid
    // encrypting it again. This way the data that has not changed isn't changed in its
    // encrypted form.
    let mut d = Recorder::default();
    treediff::diff(&previous_data, &edited_data, &mut d);
    let mut to_encrypt_data = edited_data.clone();
    for d in d.calls {
        if let ChangeType::Unchanged(keys, _) = d {
            debug!("keeping unchanged key: {:?}", keys);
            let v = yaml_get(&input_data, &keys)?;
            to_encrypt_data.set(&keys, v);
        }
    }

    // encrypt the data with the recipients
    let output_data = encrypt_yaml(&to_encrypt_data, &recipients)?;
    // save the encrypted data in the original file
    let output = File::create(&args.file).path_ctx(&args.file)?;
    serde_yaml::to_writer(output, &output_data)?;

    Ok(())
}

fn yaml_get<'a>(data: &'a serde_yaml::Value, keys: &[Key]) -> Result<&'a serde_yaml::Value> {
    if keys.is_empty() {
        return Ok(data);
    }
    let key = &keys[0];
    match key {
        Key::String(k) => {
            let value = data.get(k).ok_or(AppError::KeyNotFoundError)?;
            yaml_get(value, &keys[1..])
        }
        Key::Index(i) => {
            let value = data.get(i).ok_or(AppError::KeyNotFoundError)?;
            yaml_get(value, &keys[1..])
        }
    }
}
