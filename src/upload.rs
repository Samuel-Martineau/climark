use crate::error::ClimarkError;
use crowdmark::Client;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{RenderSettings, render};
use jpeg_encoder::{ColorType, Encoder};
use std::io::{self, Read as _};
use std::sync::Arc;

pub async fn upload_assessment(
    client: Client,
    assessment_id: &str,
    nosubmit: bool,
) -> Result<(), ClimarkError> {
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|_e| ClimarkError::StdinRead)?;
    let data = Arc::new(buffer);
    let pdf = Pdf::new(data).map_err(|_e| ClimarkError::PdfParse)?;
    let interpreter_settings = InterpreterSettings::default();
    let render_settings = RenderSettings {
        x_scale: 2.0,
        y_scale: 2.0,
        bg_color: WHITE,
        ..Default::default()
    };

    let pages = pdf
        .pages()
        .iter()
        .map(|page| {
            let pixmap = render(page, &interpreter_settings, &render_settings);
            let width = pixmap.width();
            let height = pixmap.height();

            let pixels = pixmap.take_unpremultiplied();

            let rgb: Vec<u8> = pixels.iter().flat_map(|p| [p.r, p.g, p.b]).collect();

            let mut jpeg_data = Vec::new();
            let encoder = Encoder::new(&mut jpeg_data, 70);
            encoder.encode(&rgb, width, height, ColorType::Rgb).unwrap();
            jpeg_data
        })
        .enumerate();

    let csrf = client.get_csrf().await?;
    client
        .upload_assessment(&csrf, assessment_id, pages)
        .await?;
    if !nosubmit {
        client.submit_assessment(&csrf, assessment_id).await?;
    }
    Ok(())
}
