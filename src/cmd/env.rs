use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use clap::Args;
use serde_yaml as sy;

use crate::cli::ENV_PATH_SEP;
use crate::error::{Result, YageError};
use crate::{decrypt_yaml, load_identities, stdin_or_file};

/// Execute a command with the environment from the encrypted YAML file
///
/// The YAML file must contain a map with string keys and values. The keys are the environment
/// variable names, and the values are the environment variable values.
/// Other more complex YAML structures are not supported.
#[derive(Args, Debug)]
pub struct EnvArgs {
    /// Start with an empty environment
    #[clap(short, long, default_value_t = false)]
    pub ignore_environment: bool,

    /// Decrypt with the specified key
    ///
    /// Note that passing private keys as arguments or environment variables may expose them to other users
    /// on the system, and store them in your shell history. As a consequence the --key option and YAGE_KEY
    /// environment variable should only be used in a secure environment.
    ///
    /// May be repeated.
    ///
    /// Multiple values may be passed in the YAGE_KEY environment variable separated by commas.
    #[clap(
        short,
        long = "key",
        name = "KEY",
        env = "YAGE_KEY",
        value_delimiter = ','
    )]
    pub keys: Vec<String>,

    /// Decrypt with the key at PATH
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

    /// The YAML file to decrypt
    #[arg()]
    pub file: PathBuf,

    /// The command to run
    #[arg()]
    pub command: String,

    /// The command arguments
    #[arg()]
    pub args: Vec<String>,
}

pub fn env(args: &EnvArgs) -> Result<i32> {
    let identities = load_identities(&args.keys, &args.key_files)?;
    debug!("loading yaml file: {:?}", args.file);
    let input_data: sy::Value = sy::from_reader(stdin_or_file(&args.file)?)?;
    let output_data = decrypt_yaml(&input_data, &identities)?;
    let env_data = build_env(&output_data)?;
    for (key, value) in &env_data {
        debug!("{key}={value}");
    }
    let mut command = Command::new(&args.command);
    if args.ignore_environment {
        command.env_clear();
    }
    command.args(&args.args).envs(&env_data);
    let status = command.spawn()?.wait()?;
    Ok(status.code().unwrap_or(1))
}

fn build_env(data: &sy::Value) -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();
    match data {
        sy::Value::Mapping(mapping) => {
            for (key, value) in mapping {
                let key = plain_value_to_string(key)?;
                let value = plain_value_to_string(value)?;
                env.insert(key, value);
            }
        }
        _ => Err(YageError::NotAMap)?,
    }
    Ok(env)
}

fn plain_value_to_string(data: &sy::Value) -> Result<String> {
    Ok(match data {
        sy::Value::String(s) => s.to_owned(),
        sy::Value::Number(n) => {
            if n.is_f64() {
                n.as_f64().unwrap().to_string()
            } else if n.is_i64() {
                n.as_i64().unwrap().to_string()
            } else {
                n.as_u64().unwrap().to_string()
            }
        }
        _ => Err(YageError::NotAStringOrNumber)?,
    })
}
