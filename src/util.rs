use std::fs::File;
use std::io::{stdin, stdout, BufReader, Read, Write};
use std::path::Path;

use crate::error::{IOResultExt, Result};

pub fn stdout_or_file(path: &Path) -> Result<Box<dyn Write>> {
    Ok(if path == Path::new("-") {
        Box::new(stdout())
    } else {
        Box::new(File::create(path).path_ctx(path)?)
    })
}

pub fn stdin_or_file(path: &Path) -> Result<BufReader<Box<dyn Read>>> {
    Ok(if path == Path::new("-") {
        BufReader::new(Box::new(stdin()))
    } else {
        BufReader::new(Box::new(File::open(path).path_ctx(path)?))
    })
}
