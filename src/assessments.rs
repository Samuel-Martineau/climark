use crate::error::ClimarkError;

use tabled::{
    builder::Builder,
    settings::{
        Color,
        object::{Columns, Object, Rows},
    },
};
pub async fn list_assessments(
    client: crowdmark::Client,
    course_id: &str,
    json: &bool,
) -> Result<(), ClimarkError> {
    let assessments = client.list_assessments(course_id).await?;

    if *json {
        println!("{}", serde_json::to_string(&assessments).unwrap());
    } else {
        let mut builder = Builder::new();
        builder.push_record(["ID", "Title", "Score (%)", "Due"]);
        for assessment in assessments {
            builder.push_record([
                assessment.id,
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
        let mut table = crate::make_table(builder);
        table.modify(Columns::one(0).not(Rows::one(0)), Color::FG_GREEN);
        table.modify(Columns::one(1).not(Rows::one(0)), Color::FG_BLUE);
        table.modify(Columns::one(2).not(Rows::one(0)), Color::FG_YELLOW);
        table.modify(Columns::one(3).not(Rows::one(0)), Color::FG_MAGENTA);
        println!("{table}");
    }
    Ok(())
}
