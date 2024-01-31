use crate::cli::EncryptArgs;
use crate::error::Result;

pub fn encrypt(args: &EncryptArgs) -> Result<()> {
    info!("Encrypting a file {args:?}");
    Ok(())
}
