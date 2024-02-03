use std::{fs, io::Write};

include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    generate_man::<Cli>()?;
    generate_markdown::<Cli>()?;
    Ok(())
}

fn generate_man<C: clap::CommandFactory>() -> std::io::Result<()> {
    let command = C::command();
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

fn generate_markdown<C: clap::CommandFactory>() -> std::io::Result<()> {
    let md = clap_markdown::help_markdown::<C>();
    let mut f = fs::File::create("doc/CommandLineHelp.md")?;
    write!(f, "{md}")?;
    Ok(())
}
