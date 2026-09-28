use crate::ffmpeg::ffmpeg_with_args;

pub fn extract_frames(
    file_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ffmpeg_with_args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        file_path,
        output_path,
    ]);

    let _ = cmd.status()?;
    Ok(())
}