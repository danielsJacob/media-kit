use crate::ffmpeg::ffmpeg_with_args;

pub fn download_url(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ffmpeg_with_args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        url,
        "-c:v",
        "copy",
        "-c:a",
        "copy",
        output_path,
    ]);

    let _status = cmd.status()?;

    Ok(())
}