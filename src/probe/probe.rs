use crate::ffmpeg::ffprobe_command;
use std::process::Stdio;

pub fn probe_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = ffprobe_command();

    cmd.arg(file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output()?;

    if !output.status.success() {
        return Err("ffprobe failed".into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn get_stream_info(
    file_path: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut cmd = ffprobe_command();

    cmd.args([
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_streams",
        "-show_format",
        file_path,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    let output = cmd.output()?;

    if !output.status.success() {
        return Err("ffprobe failed".into());
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

pub fn get_duration(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let data = get_stream_info(file_path)?;

    data.get("format")
        .and_then(|fmt| fmt.get("duration"))
        .and_then(|d| d.as_f64())
        .ok_or_else(|| "no duration found".into())
}
//fix
pub fn _get_video_codec(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = get_stream_info(file_path)?;

    data.get("streams")
        .and_then(|s| s.as_array())
        .and_then(|streams| {
            streams.iter().find(|s| {
                s.get("codec_type").and_then(|t| t.as_str()) == Some("video")
            })
        })
        .and_then(|s| s.get("codec_name"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "video codec not found".into())
}
//fix
pub fn _get_audio_codec(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = get_stream_info(file_path)?;

    data.get("streams")
        .and_then(|s| s.as_array())
        .and_then(|streams| {
            streams.iter().find(|s| {
                s.get("codec_type").and_then(|t| t.as_str()) == Some("audio")
            })
        })
        .and_then(|s| s.get("codec_name"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "audio codec not found".into())
}