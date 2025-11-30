use crate::OutputFormat;
use crate::error::ClimarkError;
use comfy_table::{Attribute::Bold, Cell, Color, Table};

pub async fn list_assessments(
    client: crowdmark::Client,
    course_id: &str,
    format: &OutputFormat,
) -> Result<(), ClimarkError> {
    let assessments = client.list_assessments(course_id).await?;

    match *format {
        OutputFormat::Json => println!("{}", serde_json::to_string(&assessments).unwrap()),
        OutputFormat::Plain => {
            for assessment in assessments {
                println!("{}\t{}", assessment.id, assessment.title);
            }
        }
        OutputFormat::Pretty => {
            let mut table = Table::new();
            table.load_preset(crate::TABLE_PRESET).set_header(vec![
                Cell::new("ID").add_attribute(Bold),
                Cell::new("Title").add_attribute(Bold),
                Cell::new("Score (%)").add_attribute(Bold),
                Cell::new("Due").add_attribute(Bold),
            ]);
            for assessment in assessments {
                let score = assessment
                    .score
                    .map(|s| format!("{:>3.0}", s * 100.0))
                    .unwrap_or_default();
                let due = assessment
                    .graded
                    .map(|g| {
                        g.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();

                table.add_row([
                    Cell::new(&assessment.id).fg(Color::Green),
                    Cell::new(&assessment.title).fg(Color::Blue),
                    Cell::new(score).fg(Color::Magenta),
                    Cell::new(due).fg(Color::Yellow),
                ]);
            }
            println!("{table}");
        }
    }

    Ok(())
}
