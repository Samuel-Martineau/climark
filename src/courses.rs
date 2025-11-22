use crate::OutputFormat;
use crate::error::ClimarkError;
use crate::make_table;

use tabled::{
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
    },
};
pub async fn list_courses(
    client: crowdmark::Client,
    format: &OutputFormat,
) -> Result<(), ClimarkError> {
    let courses = client.list_courses().await?;
    match *format {
        OutputFormat::Pretty => {
            let mut builder = Builder::new();
            builder.push_record(["Name", "ID", "Assessments"]);

            let mut last = 0;
            for (index, course) in courses.into_iter().enumerate() {
                builder.push_record([course.name, course.id, course.assessment_count.to_string()]);
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
    Ok(())
}
