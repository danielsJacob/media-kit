use crate::ffmpeg::ffmpeg_with_args;
use std::process::Stdio;

pub fn _build_output_path(file_path: &str, container: &str) -> String {
    let path = std::path::Path::new(file_path);
    let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    dir.join(format!("{}-remuxed.{}", name, container))
        .to_string_lossy()
        .to_string()
}

pub async fn _remux_file(
    file_path: String,
    output_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let _result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let status = ffmpeg_with_args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            &file_path,
            "-c",
            "copy",
            "-map",
            "0",
            &output_path,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("ffmpeg error: {}", e))?
        .success();

        if status {
            Ok(())
        } else {
            Err("remux failed".into())
        }
    })
    .await
    .map_err(|e| format!("task error: {}", e))??;

    Ok(())
}
