use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use serde_yaml as sy;
use tempfile::tempdir;

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
    #[clap(short, long = "key", value_name = "KEY", env = "YAGE_KEY", value_delimiter = ',')]
    pub keys: Vec<String>,

    /// Decrypt with the key at in this file
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_KEY_FILE environment variable separated by the system path separator.
    #[clap(
        short = 'K',
        long = "key-file",
        value_name = "FILE",
        env = "YAGE_KEY_FILE",
        value_delimiter = ENV_PATH_SEP,
    )]
    pub key_files: Vec<PathBuf>,

    /// The encrypted YAML file to edit
    #[arg()]
    pub file: PathBuf,
}

pub fn edit(args: &EditArgs) -> Result<i32> {
    if args.file == Path::new("-") {
        return Err(YageError::InPlaceStdin);
    }
    let identities = load_identities(&args.keys, &args.key_files)?;
    if identities.is_empty() {
        return Err(YageError::NoKeys);
    }
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
    // Find what has not changed, and keep those values from the original
    // encrypted file unchanged. That data is encrypted with a nonce that
    // makes it appear different every time it is encrypted, so we avoid
    // encrypting it again. This way the data that has not changed isn't
    // changed in its encrypted form.
    let mut to_encrypt_data = edited_data.clone();
    apply_unchanged(&previous_data, &edited_data, &input_data, &mut to_encrypt_data)?;

    let output_data = encrypt_yaml(&to_encrypt_data, &recipients)?;
    write_yaml(&args.file, &output_data)?;
    Ok(0)
}

/// Recursively walk two value trees in tandem. For paths where the previous
/// and edited values are equal, inject the original (encrypted) value from
/// `original` into `target`.
fn apply_unchanged(
    previous: &sy::Value,
    edited: &sy::Value,
    original: &sy::Value,
    target: &mut sy::Value,
) -> Result<()> {
    apply_unchanged_rec(previous, edited, original, target)
}

fn apply_unchanged_rec(
    previous: &sy::Value,
    edited: &sy::Value,
    original: &sy::Value,
    target: &mut sy::Value,
) -> Result<()> {
    if previous == edited {
        return Ok(());
    }
    if let sy::Value::Mapping(prev_map) = previous
        && let sy::Value::Mapping(edit_map) = edited
        && let sy::Value::Mapping(orig_map) = original
        && let sy::Value::Mapping(target_map) = target
    {
        return apply_unchanged_mapping(prev_map, edit_map, orig_map, target_map);
    }
    if let sy::Value::Sequence(prev_seq) = previous
        && let sy::Value::Sequence(edit_seq) = edited
        && let sy::Value::Sequence(orig_seq) = original
        && let sy::Value::Sequence(target_seq) = target
    {
        return apply_unchanged_sequence(prev_seq, edit_seq, orig_seq, target_seq);
    }
    Ok(())
}

fn apply_unchanged_mapping(
    prev_map: &sy::Mapping,
    edit_map: &sy::Mapping,
    orig_map: &sy::Mapping,
    target_map: &mut sy::Mapping,
) -> Result<()> {
    for (key, prev_val) in prev_map {
        if let Some(edit_val) = edit_map.get(key)
            && let (Some(orig_val), Some(target_val)) = (orig_map.get(key), target_map.get_mut(key))
        {
            if prev_val == edit_val {
                *target_val = orig_val.clone();
            } else {
                apply_unchanged_rec(prev_val, edit_val, orig_val, target_val)?;
            }
        }
    }
    Ok(())
}

fn apply_unchanged_sequence(
    prev_seq: &sy::Sequence,
    edit_seq: &sy::Sequence,
    orig_seq: &sy::Sequence,
    target_seq: &mut sy::Sequence,
) -> Result<()> {
    let len = prev_seq.len().min(edit_seq.len()).min(target_seq.len()).min(orig_seq.len());
    for i in 0..len {
        if prev_seq[i] == edit_seq[i] {
            target_seq[i] = orig_seq[i].clone();
        } else {
            apply_unchanged_rec(&prev_seq[i], &edit_seq[i], &orig_seq[i], &mut target_seq[i])?;
        }
    }
    Ok(())
}

fn run_editor(editor: &str, temp_file: &std::path::Path) -> Result<()> {
    let editor_process_res = Command::new(editor).arg(temp_file).spawn();
    let mut editor_process = match editor_process_res {
        Ok(ep) => ep,
        Err(err) => {
            // if we can't use the editor string as a command, it may have arguments that we need to split
            if let Some(ref editor_args) = shlex::split(editor) {
                if editor_args.is_empty() {
                    // we need at least one element, fallback to the previous error so that the user
                    // can see its editor value in the error message
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
