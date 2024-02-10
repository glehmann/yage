use std::fs::File;
use std::process::Command;

use serde_yaml as sy;
use tempfile::tempdir;
use treediff::tools::{ChangeType, Recorder};
use treediff::value::Key;
use treediff::Mutable;

use crate::cli::EditArgs;
use crate::error::{IOResultExt, Result, YageError};
use crate::{create_private_file, decrypt_yaml, encrypt_yaml, load_identities, load_recipients};

pub fn edit(args: &EditArgs) -> Result<i32> {
    let identities = load_identities(&args.keys, &args.key_files)?;
    let mut recipients = load_recipients(&args.recipients, &args.recipient_files)?;
    // add the identities to the recipients, so that the user can edit the file
    for identity in &identities {
        recipients.push(identity.to_public());
    }
    debug!("loading yaml file: {:?}", args.file);
    let input_data: sy::Value = sy::from_reader(File::open(&args.file).path_ctx(&args.file)?)?;
    let previous_data = decrypt_yaml(&input_data, &identities)?;
    // save the decrypted data in an editable temporary file. The file has the same name as the
    // original file, but in a temporary directory. This way the user knows which file he is
    // editing if its editor shows the file name.
    let dir = tempdir()?;
    let filename = args.file.file_name().ok_or(YageError::InvalidFileName {
        path: args.file.clone(),
    })?;
    let temp_file = dir.path().join(filename);
    {
        let output = create_private_file(&temp_file)?;
        sy::to_writer(output, &previous_data)?;
    }
    // open the editor
    let editor_process_res = Command::new(&args.editor).arg(&temp_file).spawn();
    let mut editor_process = match editor_process_res {
        Ok(ep) => ep,
        Err(err) => {
            // if we can't usse the editor string as a command, it may have arguments that we need to split
            if let Some(ref editor_args) = shlex::split(&args.editor) {
                if editor_args.is_empty() {
                    // fallback to the previous error
                    return Err(err).path_ctx(&args.editor);
                }
                Command::new(&editor_args[0])
                    .args(&editor_args[1..])
                    .arg(&temp_file)
                    .spawn()
                    .path_ctx(&editor_args[0])?
            } else {
                // we can't split the editor string, so fallback to the previous error
                return Err(err).path_ctx(&args.editor);
            }
        }
    };
    if !editor_process.wait()?.success() {
        return Err(YageError::Editor);
    }
    // load the data edited by the user
    let edited_data: sy::Value = sy::from_reader(File::open(&temp_file)?)?;

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
    sy::to_writer(output, &output_data)?;

    Ok(0)
}

fn yaml_get<'a>(data: &'a sy::Value, keys: &[Key]) -> Result<&'a sy::Value> {
    if keys.is_empty() {
        return Ok(data);
    }
    match &keys[0] {
        Key::String(k) => {
            let k: sy::Value = sy::from_str(k)?;
            let value = data.get(k).ok_or(YageError::KeyNotFound)?;
            yaml_get(value, &keys[1..])
        }
        Key::Index(i) => {
            let value = data.get(i).ok_or(YageError::KeyNotFound)?;
            yaml_get(value, &keys[1..])
        }
    }
}
