use std::fs::File;
use std::io::{stdout, Write};
use std::path::Path;

use crate::error::{IOResultExt, Result};

pub fn stdout_or_file(path: &Path) -> Result<Box<dyn Write>> {
    Ok(if path == Path::new("-") {
        Box::new(stdout()) as Box<dyn Write>
    } else {
        Box::new(File::create(path).path_ctx(path)?) as Box<dyn Write>
    })
}
