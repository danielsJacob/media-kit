use crate::ffmpeg::ffmpeg_with_args;

pub fn _build_output_path(file_path: &str, fmt: &str) -> String {
    let path = std::path::Path::new(file_path);
    let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    dir.join(format!("{}-fmt.{}", name, fmt))
        .to_string_lossy()
        .to_string()
}

pub fn _reformat(
    file_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = ffmpeg_with_args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        file_path,
        "-c",
        "copy",
        output_path,
    ])
    .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("reformat failed".into())
    }
}
