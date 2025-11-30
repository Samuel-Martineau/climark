mod assessments;
mod cli;
mod courses;
mod error;
mod login;
mod upload;

use clap::Parser;
use cli::{Cli, Commands, OutputFormat};
use error::ClimarkError;

pub const TABLE_PRESET: &str = "    ────           ";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let token = match &cli.crowdmark_session_token {
        Some(t) => t,
        None => &login::get_token().await.unwrap(),
    };

    let client = crowdmark::Client::new(token).unwrap();

    match &cli.command {
        Commands::ListCourses { format, silent } => {
            handle_error(courses::list_courses(client, format).await, *silent);
        }
        Commands::ListAssessments {
            course_id,
            format,
            silent,
        } => handle_error(
            assessments::list_assessments(client, course_id, format).await,
            *silent,
        ),
        Commands::Login => handle_error(login::login().await, false),
        Commands::UploadAssessment {
            ids,
            silent,
            submit,
        } => handle_error(
            upload::upload_assessment(client, ids.last().unwrap(), submit).await,
            *silent,
        ),
    }
}

fn handle_error(result: Result<(), ClimarkError>, silent: bool) {
    if let Err(e) = result
        && !silent
    {
        eprintln!("Error: {e}");
    }
}
