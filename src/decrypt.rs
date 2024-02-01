use crate::cli::DecryptArgs;
use crate::error::Result;
use crate::util::{decrypt_yaml, load_identities, stdin_or_file, stdout_or_file};

pub fn decrypt(args: &DecryptArgs) -> Result<()> {
    let identities = load_identities(&args.keys, &args.key_files)?;
    debug!("loading yaml file: {:?}", args.file);
    let input_data: serde_yaml::Value = serde_yaml::from_reader(stdin_or_file(&args.file)?)?;
    let output_data = decrypt_yaml(&input_data, &identities)?;
    let output = stdout_or_file(if args.inplace {
        &args.file
    } else {
        &args.output
    })?;
    serde_yaml::to_writer(output, &output_data)?;
    Ok(())
}
