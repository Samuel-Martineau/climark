use crowdmark::error::CrowdmarkError;
use hayro::{Pdf, RenderSettings, render};
use hayro_interpret::InterpreterSettings;
use std::io::{self, Read};
use std::sync::Arc;

pub fn upload_assessment(course_id: &str, assessment_id: &str) -> Result<(), CrowdmarkError> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();
    let data = Arc::new(buffer);
    let pdf = Pdf::new(data).unwrap();
    let interpreter_settings = InterpreterSettings::default();
    let render_settings = RenderSettings {
        x_scale: 1.0,
        y_scale: 1.0,
        ..Default::default()
    };

    for (idx, page) in pdf.pages().iter().enumerate() {
        let png = render(page, &interpreter_settings, &render_settings).take_png();
        // Do something with png
        // let mut stdout = io::stdout().lock();
        // stdout.write_all(&png).unwrap();
    }
    Ok(())
}
