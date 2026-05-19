mod assessments;
mod cli;
mod courses;
mod error;
mod login;
mod upload;

use clap::Parser as _;
use cli::{Cli, Commands, OutputFormat};
use error::ClimarkError;

pub const TABLE_PRESET: &str = "    \u{2500}\u{2500}\u{2500}\u{2500}           ";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let token = match cli.crowdmark_session_token {
        Some(t) => t,
        None => login::get_token().await,
    };

    let client = crowdmark::Client::new(&token).expect("Failed to initialize Crowdmark client");

    match cli.command {
        Commands::ListCourses { format, silent } => {
            handle_error(courses::list_courses(client, &format).await, silent);
        }
        Commands::ListAssessments {
            course_id,
            format,
            silent,
        } => handle_error(
            assessments::list_assessments(client, &course_id, &format).await,
            silent,
        ),
        Commands::Login => login::login().await,
        Commands::UploadAssessment {
            ids,
            scale,
            silent,
            nosubmit,
        } => handle_error(
            upload::upload_assessment(
                client,
                ids.last().expect("No assignment/course ID provided!"),
                scale,
                nosubmit,
            )
            .await,
            silent,
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
