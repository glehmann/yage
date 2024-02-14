use std::path::PathBuf;

use age::secrecy::ExposeSecret;
use age::x25519::Identity;
use clap::Args;

use crate::error::{IOResultExt, Result};
use crate::{stdout_or_file, stdout_or_private_file};

/// Generate a new age key
///
/// The public part of the key is logged to the standard error output. It may be computed from the private
/// key with the pubkey command.
///
/// The key is written in the age format, which is compatible with the age tool.
#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// The output path to the private key file
    ///
    /// The private key is written to the standard output by default.
    #[clap(short, long, default_value = "-")]
    pub output: PathBuf,

    /// The output path to the public key file
    #[clap(short, long)]
    pub public: Option<PathBuf>,
}

pub fn keygen(args: &KeygenArgs) -> Result<i32> {
    let key = Identity::generate();
    let mut output = stdout_or_private_file(&args.output)?;
    writeln!(output, "{}", key.to_string().expose_secret()).path_ctx(&args.output)?;
    info!("Public key: {}", key.to_public());
    if let Some(ref public) = args.public {
        let mut output = stdout_or_file(public)?;
        writeln!(output, "{}", key.to_public()).path_ctx(public)?;
    }
    Ok(0)
}
