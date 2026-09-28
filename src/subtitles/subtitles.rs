use crate::ffmpeg::ffprobe_binary_path;
use std::io;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum SubtitleFormat {
    Srt,
    Ass,
    Vtt,
}

impl SubtitleFormat {
    pub fn codec(self) -> &'static str {
        match self {
            Self::Srt => "srt",
            Self::Ass => "ass",
            Self::Vtt => "webvtt",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Srt => "srt",
            Self::Ass => "ass",
            Self::Vtt => "vtt",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubtitleTrack {
    pub index: usize,
    pub language: String,
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct FfprobeOutput {
    pub streams: Vec<FfprobeStream>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FfprobeStream {
    pub index: usize,

    #[serde(default)]
    pub tags: Option<FfprobeTags>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FfprobeTags {
    #[serde(default)]
    pub language: Option<String>,

    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    FileInput,
    FormatSelect,
    TrackSelect,
}

pub fn extract_subtitles(
    file_path: &str,
    format_type: SubtitleFormat,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = crate::ffmpeg::ffmpeg_with_args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        file_path,
        "-map",
        "0:s:0",
        "-c:s",
        format_type.codec(),
        output_path,
    ]);

    let _status = cmd.status()?;

    Ok(())
}

pub fn get_subtitle_tracks(file: &str) -> io::Result<Vec<SubtitleTrack>> {
    let ffprobe = ffprobe_binary_path();

    let output = Command::new(ffprobe)
        .args([
            "-v",
            "quiet",
            "-select_streams",
            "s",
            "-show_entries",
            "stream=index:stream_tags=language,title",
            "-of",
            "json",
            file,
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "ffprobe failed"));
    }

    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    Ok(parsed
        .streams
        .into_iter()
        .map(|stream| SubtitleTrack {
            index: stream.index,

            language: stream
                .tags
                .as_ref()
                .and_then(|tags| tags.language.clone())
                .unwrap_or_default(),

            title: stream
                .tags
                .as_ref()
                .and_then(|tags| tags.title.clone())
                .unwrap_or_default(),
        })
        .collect())
}
