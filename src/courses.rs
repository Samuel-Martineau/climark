use crate::OutputFormat;
use crate::error::ClimarkError;
use comfy_table::{Attribute::Bold, Cell, Color, Table};

const GREY: Color = Color::Rgb {
    r: 128,
    g: 128,
    b: 128,
};

pub async fn list_courses(
    client: crowdmark::Client,
    format: &OutputFormat,
) -> Result<(), ClimarkError> {
    let courses = client.list_courses().await?;
    match *format {
        OutputFormat::Pretty => {
            let mut table = Table::new();
            table.load_preset(crate::TABLE_PRESET).set_header(vec![
                Cell::new("Name").add_attribute(Bold),
                Cell::new("ID").add_attribute(Bold),
                Cell::new("Assessments").add_attribute(Bold),
            ]);
            for course in courses {
                let (name_colour, id_colour, assessment_colour) = if course.archived {
                    (GREY, GREY, GREY)
                } else {
                    (Color::Green, Color::Blue, Color::Yellow)
                };

                table.add_row([
                    Cell::new(&course.name).fg(name_colour),
                    Cell::new(&course.id).fg(id_colour),
                    Cell::new(course.assessment_count).fg(assessment_colour),
                ]);
            }
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
