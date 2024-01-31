use age::secrecy::ExposeSecret;
use age::x25519::Identity;

use crate::cli::KeygenArgs;
use crate::error::{IOResultExt, Result};
use crate::util::stdout_or_file;

pub fn keygen(args: &KeygenArgs) -> Result<()> {
    let mut output = stdout_or_file(&args.output)?;
    let key = Identity::generate();
    info!("Public key: {}", key.to_public());
    writeln!(output, "{}", key.to_string().expose_secret()).path_ctx(&args.output)?;
    Ok(())
}
