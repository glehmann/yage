use age::secrecy::ExposeSecret;
use age::x25519::Identity;

use crate::cli::KeygenArgs;
use crate::error::{IOResultExt, Result};
use crate::{stdout_or_file, stdout_or_private_file};

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
