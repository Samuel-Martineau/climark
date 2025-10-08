include!("src/cli.rs");
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Fish};
use std::io::{Error, Write};

fn main() -> Result<(), Error> {
    let mut cmd = Cli::command();
    let profile = std::env::var("PROFILE").unwrap();

    let path = generate_to(Fish, &mut cmd, "climark", format!("target/{profile}"))?;

    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;

    write!(
        file,
        r#"
complete -c climark -f
complete -c climark -kn "__fish_seen_subcommand_from list-assessments" -a "(climark list-courses -s --format=plain)"
complete -c climark -kn "__fish_seen_subcommand_from upload-assessment" -a "(climark list-assessments -s --format=plain)"\
"#
    )?;

    Ok(())
}
