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
    hide_scores: &bool,
    json: &bool,
) -> Result<(), ClimarkError> {
    let assessments = client.list_assessments(course_id).await?;

    if *json {
        println!("{}", serde_json::to_string(&assessments).unwrap());
    } else {
        let mut builder = Builder::new();

        let mut header = vec!["ID".to_string(), "Title".to_string()];
        if !*hide_scores {
            header.push("Score (%)".to_string());
        }
        header.push("Due".to_string());

        builder.push_record(header);
        assessments
            .iter()
            .map(|a| {
                let mut row = vec![a.id.clone(), a.title.clone()];

                if !*hide_scores {
                    row.push(
                        a.score
                            .map(|s| format!("{:>3.0}", s * 100.0))
                            .unwrap_or_default(),
                    );
                }

                row.push(
                    a.graded
                        .map(|g| {
                            g.with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        })
                        .unwrap_or_default(),
                );

                row
            })
            .for_each(|row| builder.push_record(row));

        let mut table = crate::make_table(builder);
        table.modify(Columns::one(0).not(Rows::one(0)), Color::FG_GREEN);
        table.modify(Columns::one(1).not(Rows::one(0)), Color::FG_BLUE);
        table.modify(Columns::one(2).not(Rows::one(0)), Color::FG_MAGENTA);

        if !*hide_scores {
            table.modify(Columns::one(3).not(Rows::one(0)), Color::FG_YELLOW);
        }
        println!("{table}");
    }
    Ok(())
}
