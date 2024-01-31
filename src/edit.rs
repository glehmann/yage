use crate::cli::EditArgs;
use crate::error::Result;

pub fn edit(args: &EditArgs) -> Result<()> {
    info!("editing a file {args:?}");
    Ok(())
}
