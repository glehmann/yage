use serde_yaml as sy;

use crate::cli::DecryptArgs;
use crate::error::Result;
use crate::util::{decrypt_yaml, load_identities, stdin_or_file, stdout_or_file};

pub fn decrypt(args: &DecryptArgs) -> Result<()> {
    let identities = load_identities(&args.keys, &args.key_files)?;
    for file in &args.files {
        debug!("loading yaml file: {file:?}");
        let input_data: sy::Value = sy::from_reader(stdin_or_file(file)?)?;
        let output_data = decrypt_yaml(&input_data, &identities)?;
        let output = stdout_or_file(if args.in_place { file } else { &args.output })?;
        sy::to_writer(output, &output_data)?;
    }
    Ok(())
}
