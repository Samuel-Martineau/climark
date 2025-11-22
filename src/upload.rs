use crowdmark::Client;
use crowdmark::error::CrowdmarkError;
use hayro::{Pdf, RenderSettings, render};
use hayro_interpret::InterpreterSettings;
use image::ImageFormat;
use std::io::Cursor;
use std::io::{self, IsTerminal, Read};
use std::sync::Arc;

pub async fn upload_assessment(
    client: Client,
    assessment_id: &str,
    submit: &bool,
) -> Result<(), CrowdmarkError> {
    let mut buffer = Vec::new();
    if io::stdin().is_terminal() {
        println!("stdin is empty!");
        std::process::exit(1);
    }
    io::stdin().read_to_end(&mut buffer).unwrap();
    let data = Arc::new(buffer);
    let pdf = Pdf::new(data).unwrap();
    let interpreter_settings = InterpreterSettings::default();
    let render_settings = RenderSettings {
        x_scale: 1.0,
        y_scale: 1.0,
        ..Default::default()
    };

    let mut pages: Vec<(usize, Vec<u8>)> = Vec::new();

    println!("Rendering pages...");
    for (idx, page) in pdf.pages().iter().enumerate() {
        let png = render(page, &interpreter_settings, &render_settings).take_png();
        let img = image::load_from_memory(&png).unwrap();

        let mut jpeg_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageFormat::Jpeg)
            .unwrap();

        pages.push((idx + 1, jpeg_bytes));
    }

    println!("Uploading pages...");
    client.upload_assessment(assessment_id, pages).await?;
    if *submit {
        println!("Submitting assessment...");
        client.submit_assessment(assessment_id).await?;
    }
    Ok(())
}
