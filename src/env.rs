use std::collections::HashMap;
use std::process::Command;

use serde_yaml as sy;

use crate::cli::EnvArgs;
use crate::error::{AppError, Result};
use crate::util::{decrypt_yaml, load_identities, stdin_or_file};

pub fn env(args: &EnvArgs) -> Result<()> {
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
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
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
        _ => Err(AppError::NotAMapError)?,
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
        _ => Err(AppError::NotAStringOrNumberError)?,
    })
}
