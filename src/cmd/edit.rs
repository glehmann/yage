use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use tempfile::tempdir;
use yaml_edit::YamlNode;

use crate::cli::ENV_PATH_SEP;
use crate::error::{IOResultExt, Result, YageError};
use crate::{
    decrypt_yaml, encrypt_yaml, get_yaml_recipients, load_identities, map_set, read_yaml,
    read_yaml_file, replace_document_root, replace_yaml_file_document, seq_set, write_yaml,
    write_yaml_file,
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
    let (yaml_file, doc, input_data) = read_yaml_file(&args.file)?;
    let leaks = crate::check_comments_for_secrets(&yaml_file);
    for leak in &leaks {
        warn!(
            "{}:{}:{}: high-entropy token detected (z-score: {})",
            args.file.to_string_lossy(),
            leak.line,
            leak.col,
            leak.z_score,
        );
    }
    if !crate::check_recipients(&input_data) {
        warn!("{}: inconsistent recipients", args.file.to_string_lossy());
    }
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
    let to_encrypt_data = edited_data.clone();
    apply_unchanged(&previous_data, &edited_data, &input_data, &to_encrypt_data)?;

    let output_data = encrypt_yaml(&to_encrypt_data, &recipients)?;
    replace_document_root(&doc, &output_data);
    replace_yaml_file_document(&yaml_file, &doc);
    write_yaml_file(&args.file, &yaml_file)?;
    Ok(0)
}

/// Recursively walk two value trees in tandem. For paths where the previous
/// and edited values are equal, inject the original (encrypted) value from
/// `original` into `target`.
fn apply_unchanged(
    prev: &YamlNode,
    edited: &YamlNode,
    original: &YamlNode,
    target: &YamlNode,
) -> Result<()> {
    if prev.yaml_eq(edited) {
        return Ok(());
    }
    if let YamlNode::Mapping(prev_m) = prev
        && let YamlNode::Mapping(edit_m) = edited
        && let YamlNode::Mapping(orig_m) = original
        && let YamlNode::Mapping(target_m) = target
    {
        for key in prev_m.keys() {
            if let Some(edit_val) = edit_m.get(key.clone())
                && let Some(orig_val) = orig_m.get(key.clone())
                && let Some(prev_val) = prev_m.get(key.clone())
            {
                if prev_val.yaml_eq(&edit_val) {
                    map_set(target_m, key, orig_val);
                } else if let Some(target_val) = target_m.get(key.clone()) {
                    apply_unchanged(&prev_val, &edit_val, &orig_val, &target_val)?;
                }
            }
        }
    } else if let YamlNode::Sequence(prev_s) = prev
        && let YamlNode::Sequence(edit_s) = edited
        && let YamlNode::Sequence(orig_s) = original
        && let YamlNode::Sequence(target_s) = target
    {
        let len = prev_s.len().min(edit_s.len()).min(target_s.len()).min(orig_s.len());
        for i in 0..len {
            let prev_val = prev_s.get(i).unwrap();
            let edit_val = edit_s.get(i).unwrap();
            let orig_val = orig_s.get(i).unwrap();
            if prev_val.yaml_eq(&edit_val) {
                seq_set(target_s, i, orig_val);
            } else {
                let target_val = target_s.get(i).unwrap();
                apply_unchanged(&prev_val, &edit_val, &orig_val, &target_val)?;
            }
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
