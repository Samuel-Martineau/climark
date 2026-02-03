use clap::CommandFactory as _;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, PowerShell, Zsh},
};
use std::env;
use std::fs;
use std::io::{Error, Write as _};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    const BIN_NAME: &str = env!("CARGO_PKG_NAME");
    const OUT_DIR: &str = "completions";

    fs::create_dir_all(OUT_DIR)?;
    let mut cmd = Cli::command();

    let path = generate_to(Fish, &mut cmd, BIN_NAME, OUT_DIR)?;
    let mut file = fs::OpenOptions::new().append(true).open(path)?;

    write!(
        file,
        r#"
complete -c climark -f
complete -c climark -kn "__fish_climark_using_subcommand list-assessments" -a "(climark list-courses --format=plain --silent)"
complete -c climark -kn "__fish_climark_using_subcommand upload-assessment; and test (count (commandline -opc)) -eq 2" \
    -a "(climark list-courses --format=plain --silent)"
complete -c climark -kn '__fish_climark_using_subcommand upload-assessment; and test (count (commandline -opc)) -eq 3' \
    -a "(climark list-assessments (commandline -opc)[3] --format=plain --silent)"
"#
    )?;

    generate_to(Bash, &mut cmd, BIN_NAME, OUT_DIR)?;
    generate_to(PowerShell, &mut cmd, BIN_NAME, OUT_DIR)?;
    generate_to(Zsh, &mut cmd, BIN_NAME, OUT_DIR)?;
    Ok(())
}
