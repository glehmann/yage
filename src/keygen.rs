use age::secrecy::ExposeSecret;
use age::x25519::Identity;

use crate::cli::KeygenArgs;
use crate::error::{IOResultExt, Result};
use crate::util::stdout_or_file;

pub fn keygen(args: &KeygenArgs) -> Result<()> {
    let key = Identity::generate();
    let mut output = stdout_or_file(&args.output)?;
    writeln!(output, "{}", key.to_string().expose_secret()).path_ctx(&args.output)?;
    info!("Public key: {}", key.to_public());
    if let Some(ref public) = args.public {
        let mut output = stdout_or_file(public)?;
        writeln!(output, "{}", key.to_public()).path_ctx(public)?;
    }
    Ok(())
}
