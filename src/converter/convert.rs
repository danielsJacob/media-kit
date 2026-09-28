use std::process::Stdio;
use tokio::sync::oneshot;
use crate::ffmpeg::ffmpeg_with_args;

#[derive(Debug, Clone)]
pub enum ConversionResult {
    Success(String),
    Failure(String),
}

pub fn build_output_path(file_path: &str, format: &str) -> String {
    let input_path = std::path::Path::new(file_path);
    let output_directory = input_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));

    let file_stem = input_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("converted");

    let extension = format.to_ascii_lowercase();

    output_directory
        .join(format!("{}-converted.{}", file_stem, extension))
        .to_string_lossy()
        .to_string()
}

pub async fn convert_video(
    file_path: String,
    output_path: String,
    done_tx: oneshot::Sender<ConversionResult>,
) {
    let result = tokio::task::spawn_blocking(move || {
        let status = ffmpeg_with_args([
            "-hide_banner",
            "-loglevel", "error",
            "-y",
            "-i", &file_path,
            &output_path,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(format!("Conversion complete:\n{}", output_path))
        } else {
            Err(format!("ffmpeg exited with status: {}", status))
        }
    })
    .await;

    let msg = match result {
        Ok(Ok(msg)) => ConversionResult::Success(msg),
        Ok(Err(msg)) => ConversionResult::Failure(msg),
        Err(e) => ConversionResult::Failure(format!("Task panicked: {}", e)),
    };

    let _ = done_tx.send(msg);
}