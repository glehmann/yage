include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    generate_man::<Cli>()?;
    Ok(())
}

fn generate_man<C: clap::CommandFactory>() -> std::io::Result<()> {
    let command = C::command();
    let out_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);
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
