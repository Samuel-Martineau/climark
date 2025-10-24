mod cli;
use cli::{Cli, Commands, OutputFormat};
mod upload;
use crowdmark::error::CrowdmarkError;

use clap::Parser;
use tabled::{
    builder::Builder,
    settings::{
        Color, Style,
        object::{Columns, Object, Rows},
    },
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = crowdmark::Client::new(&cli.crowdmark_session_token).unwrap();

    match &cli.command {
        Commands::ListCourses { format, silent } => {
            let courses = match client.list_courses().await {
                Ok(v) => v,
                Err(e) => {
                    if !silent {
                        handle_error(e);
                    }
                    return;
                }
            };
            match format {
                OutputFormat::Pretty => {
                    let mut builder = Builder::new();
                    builder.push_record(["Name", "ID", "Assessments"]);

                    let mut last = 0;
                    for (index, course) in courses.into_iter().enumerate() {
                        builder.push_record([
                            course.name,
                            course.id,
                            course.assessment_count.to_string(),
                        ]);
                        if !course.archived {
                            last = index;
                        }
                    }
                    let mut table = make_table(builder);
                    table.modify(
                        Columns::one(0)
                            .not(Rows::one(0))
                            .not(Rows::new((last + 2)..)),
                        Color::FG_GREEN,
                    );
                    table.modify(
                        Columns::one(1)
                            .not(Rows::one(0))
                            .not(Rows::new((last + 2)..)),
                        Color::FG_BLUE,
                    );
                    table.modify(
                        Columns::one(2)
                            .not(Rows::one(0))
                            .not(Rows::new((last + 2)..)),
                        Color::FG_YELLOW,
                    );
                    table.modify(Rows::new((last + 2)..), Color::rgb_fg(128, 128, 128));
                    println!("{table}");
                }
                OutputFormat::Plain => {
                    for course in courses {
                        println!("{}\t{}", course.id, course.name);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string(&courses).unwrap()),
            }
        }
        Commands::ListAssessments {
            course_id,
            json,
            silent,
        } => {
            let assessments = match client.list_assessments(course_id).await {
                Ok(v) => v,
                Err(e) => {
                    if !silent {
                        handle_error(e);
                    }
                    return;
                }
            };

            if *json {
                println!("{}", serde_json::to_string(&assessments).unwrap());
            } else {
                let mut builder = Builder::new();
                builder.push_record(["Title", "Score (%)", "Due"]);
                for assessment in assessments {
                    builder.push_record([
                        assessment.title,
                        assessment
                            .score
                            .map(|s| format!("{:>3.0}", s * 100.0))
                            .unwrap_or_default(),
                        assessment
                            .graded
                            .map(|g| {
                                g.with_timezone(&chrono::Local)
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string()
                            })
                            .unwrap_or_default(),
                    ]);
                }
                let mut table = make_table(builder);
                table.modify(Columns::one(0).not(Rows::one(0)), Color::FG_GREEN);
                table.modify(Columns::one(1).not(Rows::one(0)), Color::FG_BLUE);
                table.modify(Columns::one(2).not(Rows::one(0)), Color::FG_YELLOW);
                println!("{table}");
            }
        }
        Commands::UploadAssessment {
            course_id,
            assessment_id,
        } => {
            upload::upload_assessment(course_id, assessment_id).unwrap();
        }
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

fn handle_error(e: CrowdmarkError) {
    match e {
        CrowdmarkError::InvalidHeaderValue(msg) => {
            eprintln!("Invalid header value. Is the session token formatted correctly?: {msg}");
        }
        CrowdmarkError::NotAuthenticated() => {
            eprintln!("Error: Not Authenticated. Are you logged in?");
        }
        CrowdmarkError::InvalidCourseID() => {
            eprintln!("Error: Invalid Course ID");
        }
        _ => {
            eprintln!("Error: {e}");
        }
    }
}
