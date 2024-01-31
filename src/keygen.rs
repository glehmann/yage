use crate::cli::KeygenArgs;
use crate::error::Result;

pub fn keygen(args: &KeygenArgs) -> Result<()> {
    info!("Generating a new key {args:?}");
    Ok(())
}
