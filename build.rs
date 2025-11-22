include!("src/cli.rs");
use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, PowerShell, Zsh},
};
use std::io::{Error, Write};

fn main() -> Result<(), Error> {
    const BIN_NAME: &str = env!("CARGO_PKG_NAME");
    const OUT_DIR: &str = "completions";

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=crowdmark/");
    println!("cargo:rerun-if-changed=src/");

    let mut cmd = Cli::command();

    let path = generate_to(Fish, &mut cmd, BIN_NAME, OUT_DIR)?;
    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;

    write!(
        file,
        r#"
complete -c climark -f
complete -c climark -kn "__fish_seen_subcommand_from list-assessments" -a "(climark list-courses -s --format=plain)"
complete -c climark -kn "__fish_seen_subcommand_from upload-assessment" -a "(climark list-assessments -s --format=plain)"\
"#
    )?;

    generate_to(Bash, &mut cmd, BIN_NAME, OUT_DIR)?;
    generate_to(PowerShell, &mut cmd, BIN_NAME, OUT_DIR)?;
    generate_to(Zsh, &mut cmd, BIN_NAME, OUT_DIR)?;

    Ok(())
}
