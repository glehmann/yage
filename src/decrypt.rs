use crate::cli::DecryptArgs;
use crate::error::Result;

pub fn decrypt(args: &DecryptArgs) -> Result<()> {
    info!("Decrypting a file {args:?}");
    Ok(())
}
