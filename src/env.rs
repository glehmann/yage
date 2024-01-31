use crate::cli::EnvArgs;
use crate::error::Result;

pub fn env(args: &EnvArgs) -> Result<()> {
    info!("running command {args:?}");
    Ok(())
}
