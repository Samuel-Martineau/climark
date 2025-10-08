include!("src/cli.rs");
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Fish};
use std::env;
use std::io::{Error, Write};

fn main() -> Result<(), Error> {
    let Some(outdir) = env::var_os("OUT_DIR") else {
        return Ok(());
    };

    let mut cmd = Cli::command();
    let path = generate_to(Fish, &mut cmd, "climark", outdir)?;

    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;

    write!(
        file,
        r#"
complete -c climark -f
complete -c climark -kn "__fish_seen_subcommand_from list-assessments" -a "(climark list-courses --format=plain)"
complete -c climark -kn "__fish_seen_subcommand_from upload-assessment" -a "(climark list-assessments --format=plain)"\
"#
    )?;

    Ok(())
}
