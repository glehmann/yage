use crate::cli::PubkeyArgs;
use crate::error::{IOResultExt, Result};
use crate::util::{load_identities, stdout_or_file};

pub fn pubkey(args: &PubkeyArgs) -> Result<()> {
    let keys = load_identities(&args.keys, &args.key_files)?;
    let mut output = stdout_or_file(&args.output)?;
    for key in keys {
        writeln!(output, "{}", key.to_public()).path_ctx(&args.output)?;
    }
    Ok(())
}
