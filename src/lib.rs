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
    mod re_encrypt;
    mod recipients;
    pub use check::*;
    pub use decrypt::*;
    pub use edit::*;
    pub use encrypt::*;
    pub use env::*;
    pub use keygen::*;
    pub use pubkey::*;
    pub use re_encrypt::*;
    pub use recipients::*;
}

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write, stdin, stdout};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use age::x25519;
use base64::prelude::*;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use rowan::ast::AstNode;
use strum::{Display, EnumIs, EnumIter, EnumString};
use substring::Substring;
use yaml_edit::{Document, Mapping, ScalarValue, Sequence, YamlBuilder, YamlFile, YamlNode};

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
        if let Err(e) = fs_mistrust::Mistrust::new().verifier().require_file().check(path) {
            warn!("file {path:?} is not private: {e}");
        }
        br
    })
}

fn yaml_str_to_node(s: &str) -> Result<YamlNode> {
    let doc = Document::from_str(s)?;
    node_from_document(&doc)
}

fn node_from_document(doc: &Document) -> Result<YamlNode> {
    if let Some(mapping) = doc.as_mapping() {
        Ok(YamlNode::Mapping(mapping))
    } else if let Some(sequence) = doc.as_sequence() {
        Ok(YamlNode::Sequence(sequence))
    } else if let Some(scalar) = doc.as_scalar() {
        Ok(YamlNode::Scalar(scalar))
    } else {
        Err(YageError::InvalidValueEncoding)
    }
}

/// Replace the root node in a Document while preserving document-level
/// children (comments, directives, document markers). The new_root is the
/// processed YAML tree to insert in place of the old root.
pub(crate) fn replace_document_root(doc: &Document, new_root: &YamlNode) {
    use rowan::NodeOrToken;
    use yaml_edit::SyntaxKind;

    let root_kinds = [
        SyntaxKind::MAPPING,
        SyntaxKind::SEQUENCE,
        SyntaxKind::SCALAR,
        SyntaxKind::ALIAS,
        SyntaxKind::TAGGED_NODE,
    ];

    let new_syntax = match new_root {
        YamlNode::Mapping(m) => m.syntax().clone(),
        YamlNode::Sequence(s) => s.syntax().clone(),
        YamlNode::Scalar(s) => s.syntax().clone(),
        YamlNode::Alias(a) => a.syntax().clone(),
        YamlNode::TaggedNode(t) => t.syntax().clone(),
    };

    for (i, child) in doc.syntax().children_with_tokens().enumerate() {
        if let NodeOrToken::Node(n) = &child {
            if root_kinds.contains(&n.kind()) {
                doc.syntax().splice_children(i..i + 1, vec![new_syntax.into()]);
                return;
            }
        }
    }
}

/// Read a YAML file, preserving ROOT-level comments via YamlFile.
/// Returns the YamlFile, its first Document, and the root YamlNode.
pub(crate) fn read_yaml_file(path: &Path) -> Result<(YamlFile, Document, YamlNode)> {
    let mut s = String::new();
    stdin_or_file(path)?.read_to_string(&mut s)?;
    let yaml_file = YamlFile::from_str(&s)?;
    let doc = yaml_file.document().unwrap_or_default();
    let value = node_from_document(&doc)?;
    Ok((yaml_file, doc, value))
}

/// Replace the DOCUMENT node inside a YamlFile's ROOT with an updated
/// Document's syntax node. This preserves ROOT-level comments (children
/// of the ROOT node that are siblings of the DOCUMENT, such as top-level
/// comments before the first YAML key).
pub(crate) fn replace_yaml_file_document(yaml_file: &YamlFile, doc: &Document) {
    use rowan::NodeOrToken;
    use yaml_edit::SyntaxKind;

    let new_syntax = doc.syntax().clone();
    for (i, child) in yaml_file.syntax().children_with_tokens().enumerate() {
        if let NodeOrToken::Node(n) = &child {
            if n.kind() == SyntaxKind::DOCUMENT {
                yaml_file.syntax().splice_children(i..i + 1, vec![new_syntax.into()]);
                return;
            }
        }
    }
}

/// Write a YamlFile to the output path. Preserves ROOT-level comments.
pub(crate) fn write_yaml_file(path: &Path, yaml_file: &YamlFile) -> Result<()> {
    let yaml_text = yaml_file.to_string();
    let mut output = stdout_or_file(path)?;
    output.write_all(yaml_text.as_bytes())?;
    Ok(())
}

/// Create a new independent mutable cursor over the same green tree.
/// yaml-edit's Clone shares cursor state (parent/index/offset), so
/// mutating one clone affects all. This gives you a separate cursor
/// that can be mutated independently. The green tree is ref-counted
/// so this is O(1) — no data copying.
fn new_mut_cursor(value: &YamlNode) -> YamlNode {
    match value {
        YamlNode::Mapping(m) => {
            let green = m.syntax().green().into_owned();
            YamlNode::from_syntax(rowan::SyntaxNode::<yaml_edit::Lang>::new_root_mut(green))
                .unwrap()
        }
        YamlNode::Sequence(s) => {
            let green = s.syntax().green().into_owned();
            YamlNode::from_syntax(rowan::SyntaxNode::<yaml_edit::Lang>::new_root_mut(green))
                .unwrap()
        }
        YamlNode::Scalar(s) => {
            let green = s.syntax().green().into_owned();
            YamlNode::from_syntax(rowan::SyntaxNode::<yaml_edit::Lang>::new_root_mut(green))
                .unwrap()
        }
        other => other.clone(),
    }
}

/// Replace the value of a SEQUENCE_ENTRY at index `i` with `val`,
/// preserving the entry's formatting tokens (DASH, WHITESPACE)
/// and the value's internal indentation.
pub(crate) fn seq_set(seq: &Sequence, i: usize, val: YamlNode) {
    use yaml_edit::SyntaxKind;

    let new_syntax = value_syntax_node(&val);

    let children: Vec<_> = seq.syntax().children_with_tokens().collect();
    let mut item_count = 0;

    for (_entry_i, child) in children.iter().enumerate() {
        let Some(node) = child.as_node() else { continue };
        if node.kind() != SyntaxKind::SEQUENCE_ENTRY {
            continue;
        }
        if item_count != i {
            item_count += 1;
            continue;
        }

        // Found the SEQUENCE_ENTRY — find and replace its content child
        let entry_children: Vec<_> = node.children_with_tokens().collect();
        for (j, ec) in entry_children.iter().enumerate() {
            if let Some(content) = ec.as_node() {
                if matches!(
                    content.kind(),
                    SyntaxKind::SCALAR
                        | SyntaxKind::MAPPING
                        | SyntaxKind::SEQUENCE
                        | SyntaxKind::ALIAS
                        | SyntaxKind::TAGGED_NODE
                ) {
                    node.splice_children(j..j + 1, vec![new_syntax.clone().into()]);
                    return;
                }
            }
        }

        // Fallback: content node not found — YamlNode::build_content uses
        // copy_node_content which preserves indentation
        let _ = seq.set(i, val);
        return;
    }
}

/// Replace the value for `key` in `map` with `val`,
/// preserving the VALUE wrapper's formatting tokens
/// and the value's internal indentation.
pub(crate) fn map_set(map: &Mapping, key: YamlNode, val: YamlNode) {
    use yaml_edit::SyntaxKind;

    let new_syntax = value_syntax_node(&val);

    for child in map.syntax().children_with_tokens() {
        let Some(node) = child.as_node() else { continue };
        if node.kind() != SyntaxKind::MAPPING_ENTRY {
            continue;
        }

        // Find KEY child and check if it matches
        let mut key_matches = false;
        for entry_child in node.children_with_tokens() {
            let Some(key_node) = entry_child.as_node() else { continue };
            if key_node.kind() != SyntaxKind::KEY {
                continue;
            }
            // KEY has one child — the actual value node
            if let Some(content) = key_node.children().next() {
                if let Some(ek_yaml) = YamlNode::from_syntax(content) {
                    if ek_yaml.yaml_eq(&key) {
                        key_matches = true;
                    }
                }
            }
            break; // KEY found (at most one per entry)
        }
        if !key_matches {
            continue;
        }

        // Found the matching MAPPING_ENTRY — find the VALUE child
        for entry_child in node.children_with_tokens() {
            let Some(value_node) = entry_child.as_node() else { continue };
            if value_node.kind() != SyntaxKind::VALUE {
                continue;
            }

            // Within the VALUE, find the content child and replace it
            let value_children: Vec<_> = value_node.children_with_tokens().collect();
            for (j, vc) in value_children.iter().enumerate() {
                let Some(content) = vc.as_node() else { continue };
                if matches!(
                    content.kind(),
                    SyntaxKind::SCALAR
                        | SyntaxKind::MAPPING
                        | SyntaxKind::SEQUENCE
                        | SyntaxKind::ALIAS
                        | SyntaxKind::TAGGED_NODE
                ) {
                    value_node.splice_children(j..j + 1, vec![new_syntax.clone().into()]);
                    return;
                }
            }
        }

        break;
    }

    // Fallback: entry not found — YamlNode::build_content uses
    // copy_node_content which preserves indentation
    map.set(key, val);
}

/// Extract the raw syntax node for the value part of a YamlNode.
fn value_syntax_node(val: &YamlNode) -> rowan::SyntaxNode<yaml_edit::Lang> {
    match val {
        YamlNode::Scalar(s) => s.syntax().clone(),
        YamlNode::Mapping(m) => m.syntax().clone(),
        YamlNode::Sequence(s) => s.syntax().clone(),
        YamlNode::Alias(a) => a.syntax().clone(),
        YamlNode::TaggedNode(t) => t.syntax().clone(),
    }
}

pub fn decrypt_yaml(value: &YamlNode, identities: &[x25519::Identity]) -> Result<YamlNode> {
    match value {
        YamlNode::Mapping(mapping) => {
            let output = new_mut_cursor(value);
            let out_m = output.as_mapping().unwrap();
            for (key, val) in mapping {
                let decrypted = decrypt_yaml(&val, identities)?;
                if !val.yaml_eq(&decrypted) {
                    map_set(out_m, key, decrypted);
                }
            }
            Ok(output)
        }
        YamlNode::Sequence(sequence) => {
            let output = new_mut_cursor(value);
            let out_s = output.as_sequence().unwrap();
            for (i, val) in sequence.into_iter().enumerate() {
                let decrypted = decrypt_yaml(&val, identities)?;
                if !val.yaml_eq(&decrypted) {
                    seq_set(out_s, i, decrypted);
                }
            }
            Ok(output)
        }
        YamlNode::Scalar(scalar) => {
            let s = scalar.as_string();
            if YageEncodedValue::from_str(&s).is_ok() {
                decrypt_value(&s, identities)
            } else {
                Ok(YamlNode::Scalar(scalar.clone()))
            }
        }
        _ => Ok(value.clone()),
    }
}

pub fn decrypt_value(s: &str, identities: &[x25519::Identity]) -> Result<YamlNode> {
    match YageEncodedValue::from_str(s) {
        Ok(yev) => {
            // raw value -> decoded value -> decrypted value -> decompressed value -> deserialized value
            let decoded = BASE64_STANDARD.decode(yev.data)?;
            let decryptor = age::Decryptor::new(&decoded[..])?;
            if decryptor.is_scrypt() {
                return Err(YageError::PassphraseUnsupported);
            }
            let decryptor =
                decryptor.decrypt(identities.iter().map(|i| i as &dyn age::Identity))?;
            let mut decompressor = DeflateDecoder::new(decryptor);
            let mut yaml_text = String::new();
            decompressor.read_to_string(&mut yaml_text)?;
            yaml_str_to_node(&yaml_text)
        }
        Err(_) => yaml_str_to_node(s),
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
        for line in input.lines() {
            let line = line.path_ctx(key_file)?;
            let line = line.trim().to_string();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let key = x25519::Identity::from_str(&line)
                .map_err(|e| YageError::KeyParse { message: e.into() })?;
            identities.push(key);
        }
    }
    Ok(identities)
}

pub fn encrypt_yaml(value: &YamlNode, recipients: &[x25519::Recipient]) -> Result<YamlNode> {
    match value {
        YamlNode::Mapping(mapping) => {
            let output = new_mut_cursor(value);
            let out_m = output.as_mapping().unwrap();
            for (key, val) in mapping {
                let encrypted = encrypt_yaml(&val, recipients)?;
                if !val.yaml_eq(&encrypted) {
                    map_set(out_m, key, encrypted);
                }
            }
            Ok(output)
        }
        YamlNode::Sequence(sequence) => {
            let output = new_mut_cursor(value);
            let out_s = output.as_sequence().unwrap();
            for (i, val) in sequence.into_iter().enumerate() {
                let encrypted = encrypt_yaml(&val, recipients)?;
                if !val.yaml_eq(&encrypted) {
                    seq_set(out_s, i, encrypted);
                }
            }
            Ok(output)
        }
        YamlNode::Scalar(scalar) => {
            let s = scalar.as_string();
            if YageEncodedValue::from_str(&s).is_ok()
                || scalar.is_null()
                || scalar.as_bool().is_some()
            {
                Ok(YamlNode::Scalar(scalar.clone()))
            } else {
                let output = encrypt_value(value, recipients)?;
                let yaml_file = YamlBuilder::scalar(ScalarValue::plain(output.as_str())).build();
                let doc = yaml_file.document().ok_or(YageError::InvalidValueEncoding)?;
                let scalar = doc.as_scalar().ok_or(YageError::InvalidValueEncoding)?.clone();
                Ok(YamlNode::Scalar(scalar))
            }
        }
        _ => Ok(value.clone()),
    }
}

pub fn encrypt_value(value: &YamlNode, recipients: &[x25519::Recipient]) -> Result<String> {
    // yaml value -> serialized value -> compressed value -> encrypted value -> encoded value
    let yaml_text = format!("{}", value);
    let mut encrypted = vec![];
    let mut encryptor = match age::Encryptor::with_recipients(
        recipients.iter().map(|r| r as &dyn age::Recipient),
    ) {
        Err(age::EncryptError::MissingRecipients) => return Err(YageError::NoRecipients),
        r => r?,
    }
    .wrap_output(&mut encrypted)?;
    let mut compressor = DeflateEncoder::new(&mut encryptor, flate2::Compression::new(6));
    compressor.write_all(yaml_text.as_bytes())?;
    compressor.finish()?;
    encryptor.finish()?;
    // prepare the recipients list (sorted and deduplicated)
    let mut recipients: Vec<_> = recipients.iter().map(|r| r.to_string()).collect();
    recipients.sort();
    recipients.dedup();
    let yev = YageEncodedValue { data: BASE64_STANDARD.encode(&encrypted), recipients };
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
        let recipient = x25519::Recipient::from_str(recipient).map_err(|e| {
            YageError::RecipientParse { recipient: recipient.to_owned(), message: e.into() }
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
                YageError::RecipientParse { recipient: recipient.to_owned(), message: e.into() }
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

pub fn check_encrypted(value: &YamlNode) -> EncryptionStatus {
    match value {
        YamlNode::Mapping(mapping) => check_encrypted_iter(mapping.iter().map(|(_, v)| v)),
        YamlNode::Sequence(sequence) => check_encrypted_iter(sequence.values()),
        YamlNode::Scalar(scalar) => {
            if YageEncodedValue::from_str(&scalar.as_string()).is_ok() {
                EncryptionStatus::Encrypted
            } else if scalar.is_null() || scalar.as_bool().is_some() {
                EncryptionStatus::NoValue
            } else {
                EncryptionStatus::NotEncrypted
            }
        }
        _ => EncryptionStatus::NotEncrypted,
    }
}

fn check_encrypted_iter(iter: impl Iterator<Item = YamlNode>) -> EncryptionStatus {
    let mut status = EncryptionStatus::NoValue;
    for value in iter {
        match check_encrypted(&value) {
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

pub fn flatten_yage_encrypted_values(value: &YamlNode) -> Vec<YageEncodedValue> {
    match value {
        YamlNode::Mapping(mapping) => {
            mapping.iter().flat_map(|(_, v)| flatten_yage_encrypted_values(&v)).collect()
        }
        YamlNode::Sequence(sequence) => {
            sequence.values().flat_map(|v| flatten_yage_encrypted_values(&v)).collect()
        }
        YamlNode::Scalar(scalar) => match YageEncodedValue::from_str(&scalar.as_string()) {
            Ok(yev) => vec![yev],
            Err(_) => vec![],
        },
        _ => vec![],
    }
}

pub fn check_recipients(value: &YamlNode) -> bool {
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

pub fn get_yaml_recipients(value: &YamlNode) -> Result<Vec<x25519::Recipient>> {
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

pub fn read_yaml(path: &Path) -> Result<YamlNode> {
    debug!("loading yaml file: {path:?}");
    let mut s = String::new();
    stdin_or_file(path)?.read_to_string(&mut s)?;
    let value = yaml_str_to_node(&s)?;
    if !check_recipients(&value) {
        warn!("{}: inconsistent recipients", path.to_string_lossy());
    }
    Ok(value)
}

pub fn write_yaml(path: &Path, value: &YamlNode) -> Result<()> {
    debug!("writing yaml file: {path:?}");
    let yaml_text = format!("{}", value);
    let mut output = stdout_or_file(path)?;
    output.write_all(yaml_text.as_bytes())?;
    Ok(())
}
