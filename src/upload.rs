use crate::error::ClimarkError;
use crowdmark::Client;
use hayro::{Pdf, RenderSettings, render};
use hayro_interpret::InterpreterSettings;
use jpeg_encoder::{ColorType, Encoder};
use std::io::{self, Read};
use std::sync::Arc;

pub async fn upload_assessment(
    client: Client,
    assessment_id: &str,
    nosubmit: &bool,
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

    let pages = pdf
        .pages()
        .iter()
        .map(|page| {
            let pixmap = render(page, &interpreter_settings, &render_settings);

            let rgb = rgba_to_rgb(pixmap.data_as_u8_slice());
            encode_jpeg(&rgb, pixmap.width(), pixmap.height())
        })
        .enumerate();

    let csrf = client.get_csrf().await?;
    client
        .upload_assessment(&csrf, assessment_id, pages)
        .await?;
    if !*nosubmit {
        client.submit_assessment(&csrf, assessment_id).await?;
    }
    Ok(())
}

fn encode_jpeg(rgb: &[u8], width: u16, height: u16) -> Vec<u8> {
    let mut jpeg_data = Vec::new();
    let encoder = Encoder::new(&mut jpeg_data, 85);
    encoder.encode(rgb, width, height, ColorType::Rgb).unwrap();
    jpeg_data
}

fn rgba_to_rgb(pixels: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(pixels.len() / 4 * 3);

    for chunk in pixels.chunks_exact(4) {
        let (r, g, b, a) = (
            f32::from(chunk[0]),
            f32::from(chunk[1]),
            f32::from(chunk[2]),
            f32::from(chunk[3]),
        );

        if a == 0.0 {
            rgb.extend_from_slice(&[255, 255, 255]);
        } else {
            let alpha = a / 255.0;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                rgb.push((r / alpha).clamp(0.0, 255.0) as u8);
                rgb.push((g / alpha).clamp(0.0, 255.0) as u8);
                rgb.push((b / alpha).clamp(0.0, 255.0) as u8);
            }
        }
    }

    rgb
}
