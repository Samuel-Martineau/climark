mod cli;
use cli::{Cli, Commands, OutputFormat};

use clap::Parser;
use colored::Colorize;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = crowdmark::Client::new(&cli.crowdmark_session_token);

    match &cli.command {
        Commands::ListCourses { format } => {
            let courses = client.list_courses().await.unwrap();
            match format {
                OutputFormat::Pretty => {
                    println!(
                        "{:<40}{:<30}{:<12}",
                        "Name".bold(),
                        "Id".bold(),
                        "Assessments".bold()
                    );

                    println!("{}", "=".repeat(81).bold());

                    for course in courses {
                        if course.archived {
                            println!(
                                "{:<40}{:<30}{:<12}",
                                course.name.bright_black(),
                                course.id.bright_black(),
                                course.assessment_count.to_string().bright_black()
                            );
                        } else {
                            println!(
                                "{:<40}{:<30}{:<12}",
                                course.name.green(),
                                course.id.blue(),
                                course.assessment_count.to_string().yellow()
                            );
                        }
                    }
                }
                OutputFormat::Plain => {
                    for course in courses {
                        println!("{}\t{}", course.id, course.name);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string(&courses).unwrap()),
            }
        }
        Commands::ListAssessments { course_id, json } => {
            let assessments = client.list_assessments(course_id).await.unwrap();
            if *json {
                println!("{}", serde_json::to_string(&assessments).unwrap());
            } else {
                println!(
                    "{:<40}{:<30}{:<20}",
                    "Title".bold(),
                    "Score".bold(),
                    "Due".bold()
                );

                println!("{}", "=".repeat(89).bold());

                for assessment in assessments {
                    println!(
                        "{:<40}{:<30}{:<20}",
                        assessment.title.green(),
                        assessment
                            .score
                            .map(|s| format!("{:>3.0}", s * 100.0))
                            .unwrap_or_default()
                            .blue(),
                        assessment
                            .graded
                            .map(|g| g
                                .with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string())
                            .unwrap_or_default()
                            .yellow()
                    );
                }
            }
        }
    }
}
