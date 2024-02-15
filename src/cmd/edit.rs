use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use clap::Args;
use serde_yaml as sy;
use tempfile::tempdir;
use treediff::tools::{ChangeType, Recorder};
use treediff::value::Key;
use treediff::Mutable;

use crate::cli::ENV_PATH_SEP;
use crate::error::{IOResultExt, Result, YageError};
use crate::{
    decrypt_yaml, encrypt_yaml, get_yaml_recipients, load_identities, read_yaml, write_yaml,
};

/// Edit an encrypted YAML file
///
/// The file is decrypted with the specified keys and open in a text editor. The user can edit the file
/// and save it. The values are then encrypted with the same keys and the recipients, and saved in the
/// original file.
///
/// The YAML file may contain some unencrypted values, and some encrypted values. The encrypted values
/// are decrypted before the edition and all the values are encrypted after the edition.
///
/// Only the modified values are encrypted, the other values are left unchanged.
#[derive(Args, Debug)]
pub struct EditArgs {
    /// The editor command to use
    #[clap(short, long, default_value = "vim", env = "EDITOR")]
    pub editor: String,

    /// Decrypt with the specified key
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_KEY environment variable separated by commas.
    #[clap(short, long = "key", name = "KEY", env = "YAGE_KEY", value_delimiter = ',')]
    pub keys: Vec<String>,

    /// Decrypt with the key at in this file
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_KEY_FILE environment variable separated by the system path separator.
    #[clap(
        short = 'K',
        long = "key-file",
        name = "KEY_FILE",
        env = "YAGE_KEY_FILE",
        value_delimiter = ENV_PATH_SEP,
    )]
    pub key_files: Vec<PathBuf>,

    /// The encrypted YAML file to edit
    #[arg()]
    pub file: PathBuf,
}

pub fn edit(args: &EditArgs) -> Result<i32> {
    if args.file == PathBuf::from("-") {
        return Err(YageError::InPlaceStdin);
    }
    let identities = load_identities(&args.keys, &args.key_files)?;
    let input_data = read_yaml(&args.file)?;
    let recipients = get_yaml_recipients(&input_data)?;
    if recipients.is_empty() {
        return Err(YageError::NoRecipients);
    }
    let previous_data = decrypt_yaml(&input_data, &identities)?;
    // save the decrypted data in an editable temporary file. The file has the same name as the
    // original file, but in a temporary directory. This way the user knows which file he is
    // editing if its editor shows the file name.
    let dir = tempdir()?;
    #[cfg(unix)]
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    let filename =
        args.file.file_name().ok_or(YageError::InvalidFileName { path: args.file.clone() })?;
    let temp_file = dir.path().join(filename);
    write_yaml(&temp_file, &previous_data)?;

    run_editor(&args.editor, &temp_file)?;

    let edited_data = read_yaml(&temp_file)?;
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

    let output_data = encrypt_yaml(&to_encrypt_data, &recipients)?;
    write_yaml(&args.file, &output_data)?;
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

fn run_editor(editor: &str, temp_file: &std::path::Path) -> Result<()> {
    let editor_process_res = Command::new(editor).arg(temp_file).spawn();
    let mut editor_process = match editor_process_res {
        Ok(ep) => ep,
        Err(err) => {
            // if we can't use the editor string as a command, it may have arguments that we need to split
            if let Some(ref editor_args) = shlex::split(editor) {
                if editor_args.is_empty() {
                    // we need at least on element, fallback to the previous error so that the user
                    // can see its editor value in th error message
                    return Err(err).path_ctx(editor);
                }
                Command::new(&editor_args[0])
                    .args(&editor_args[1..])
                    .arg(temp_file)
                    .spawn()
                    .path_ctx(&editor_args[0])?
            } else {
                // we can't split the editor string, so fallback to the previous error
                return Err(err).path_ctx(editor);
            }
        }
    };
    if !editor_process.wait()?.success() {
        return Err(YageError::Editor);
    }
    Ok(())
}
