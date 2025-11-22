mod assessments;
mod cli;
mod courses;
mod error;
mod keyring;
mod upload;

use clap::Parser;
use cli::{Cli, Commands, OutputFormat};
use error::ClimarkError;
use tabled::{builder::Builder, settings::Style};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let token = match &cli.crowdmark_session_token {
        Some(t) => t,
        None => &keyring::get_token().await.unwrap(),
    };

    let client = crowdmark::Client::new(token).await.unwrap();

    match &cli.command {
        Commands::ListCourses { format, silent } => {
            handle_error(courses::list_courses(client, format).await, *silent);
        }
        Commands::ListAssessments {
            course_id,
            json,
            silent,
        } => handle_error(
            assessments::list_assessments(client, course_id, json).await,
            *silent,
        ),
        Commands::UploadAssessment {
            assessment_id,
            silent,
            submit,
        } => handle_error(
            upload::upload_assessment(client, assessment_id, submit).await,
            *silent,
        ),
    }
}

fn make_table(b: Builder) -> tabled::Table {
    let mut table = b.build();
    let style = Style::rounded()
        .remove_left()
        .remove_right()
        .remove_vertical()
        .remove_top()
        .remove_bottom();
    table.with(style);
    table
}

fn handle_error(result: Result<(), ClimarkError>, silent: bool) {
    if let Err(e) = result
        && !silent
    {
        eprintln!("Error: {e}");
    }
}
