use crate::ffmpeg::ffmpeg_with_args;
use std::process::Stdio;
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub enum CompressionResult {
    Success(String),
    Failure(String),
}

pub fn build_output_path(file_path: &str) -> String {
    let input_path = std::path::Path::new(file_path);
    let output_directory = input_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));

    let file_stem = input_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("compressed");

    let extension = input_path
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.trim().is_empty())
        .unwrap_or("mp4")
        .to_ascii_lowercase();

    output_directory
        .join(format!("{}-compressed.{}", file_stem, extension))
        .to_string_lossy()
        .to_string()
}

pub async fn compress_video(
    file_path: String,
    output_path: String,
    done_tx: oneshot::Sender<CompressionResult>,
) {
    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = ffmpeg_with_args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            &file_path,
            "-c:v",
            "libx264",
            "-crf",
            "28",
            "-preset",
            "medium",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            &output_path,
        ]);
        let status = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(format!("Compression complete:\n{}", output_path))
        } else {
            Err(format!("ffmpeg exited with status: {}", status))
        }
    })
    .await;

    let msg = match result {
        Ok(Ok(msg)) => CompressionResult::Success(msg),
        Ok(Err(msg)) => CompressionResult::Failure(msg),
        Err(e) => CompressionResult::Failure(format!("Task panicked: {}", e)),
    };

    let _ = done_tx.send(msg);
}
