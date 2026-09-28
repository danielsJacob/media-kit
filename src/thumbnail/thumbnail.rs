use crate::ffmpeg::ffmpeg_with_args;
use std::process::Stdio;

pub fn _build_output_path(file_path: &str) -> String {
    let path = std::path::Path::new(file_path);
    let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    dir.join(format!("{}-thumb.png", name))
        .to_string_lossy()
        .to_string()
}

pub async fn _generate_thumbnail(
    file_path: String,
    timestamp: String,
    output_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let status = ffmpeg_with_args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-ss",
            &timestamp,
            "-i",
            &file_path,
            "-vframes",
            "1",
            "-vf",
            "scale=320:-1",
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
            Err("thumbnail gen failed".into())
        }
    })
    .await
    .map_err(|e| format!("task error: {}", e))??;

    Ok(())
}

pub async fn _generate_thumbnails_batch(
    file_path: String,
    timestamps: Vec<String>,
    output_dir: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for (i, ts) in timestamps.iter().enumerate() {
        let out = format!("{}/thumb_{:03}.png", output_dir, i);
        _generate_thumbnail(file_path.clone(), ts.clone(), out.clone()).await?;
        results.push(out);
    }
    Ok(results)
}

pub fn _is_valid_timestamp(ts: &str) -> bool {
    let parts: Vec<&str> = ts.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<f64>().is_ok(),
        2 => parts[0].parse::<u32>().is_ok() && parts[1].parse::<f64>().is_ok(),
        3 => {
            parts[0].parse::<u32>().is_ok()
                && parts[1].parse::<u32>().is_ok()
                && parts[2].parse::<f64>().is_ok()
        }
        _ => false,
    }
}
