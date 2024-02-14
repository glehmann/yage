use clap::{CommandFactory, Parser};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Utility commands
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
#[command(name = "cargo xtask")]
#[command(bin_name = "cargo xtask")]
enum Command {
    /// Generate the markdown documentation
    Markdown,
    /// Generate the man page
    Manpage,
    /// Generate command line references
    CliReferences,
}

fn main() -> std::io::Result<()> {
    let args = Command::parse();
    match args {
        Command::Markdown => markdown()?,
        Command::Manpage => manpage()?,
        Command::CliReferences => {
            markdown()?;
            manpage()?;
        }
    };
    Ok(())
}

fn markdown() -> std::io::Result<()> {
    let md = clap_markdown::help_markdown::<yage::cli::Cli>();
    let mut f = fs::File::create("doc/CommandLineHelp.md")?;
    write!(f, "{md}")?;
    Ok(())
}

fn manpage() -> std::io::Result<()> {
    let command = yage::cli::Cli::command();
    let out_dir: PathBuf = "doc".into();
    let name = command.get_name();
    let name = if name == "yage" {
        "yage.1".to_owned()
    } else {
        format!("yage-{}.1", command.get_name())
    };
    let fname = out_dir.join(name);
    let man = clap_mangen::Man::new(command);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    std::fs::write(fname, buffer)?;
    Ok(())
}
