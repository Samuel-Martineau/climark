use crate::error::ClimarkError;
use crowdmark::Client;
use hayro::{Pdf, RenderSettings, render};
use hayro_interpret::InterpreterSettings;
use image::ImageFormat;
use std::io::Cursor;
use std::io::{self, Read};
use std::sync::Arc;

pub async fn upload_assessment(
    client: Client,
    assessment_id: &str,
    submit: &bool,
) -> Result<(), ClimarkError> {
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|_| ClimarkError::StdinRead())?;
    let data = Arc::new(buffer);
    let pdf = Pdf::new(data).map_err(|_| ClimarkError::PdfParse())?;
    let interpreter_settings = InterpreterSettings::default();
    let render_settings = RenderSettings {
        x_scale: 3.0,
        y_scale: 3.0,
        width: None,
        height: None,
    };

    let mut pages: Vec<(usize, Vec<u8>)> = Vec::new();

    for (idx, page) in pdf.pages().iter().enumerate() {
        let png = render(page, &interpreter_settings, &render_settings).take_png();
        let img = image::load_from_memory(&png).map_err(|_| ClimarkError::PngDecode())?;

        let mut jpeg_data = Vec::new();
        img.write_to(&mut Cursor::new(&mut jpeg_data), ImageFormat::Jpeg)
            .map_err(|_| ClimarkError::JpegEncode())?;

        pages.push((idx + 1, jpeg_data));
    }

    client.upload_assessment(assessment_id, pages).await?;
    if *submit {
        client.submit_assessment(assessment_id).await?;
    }
    Ok(())
}
