use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Mac,
    WindowsX86,
    WindowsArm64,
}

impl Platform {
    pub fn current() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") | ("linux", "x86") => Some(Self::Linux),
            ("linux", "aarch64") | ("linux", "arm") => Some(Self::Linux),
            ("macos", "x86_64") | ("macos", "x86") => Some(Self::Mac),
            ("macos", "aarch64") | ("macos", "arm") => Some(Self::Mac),
            ("windows", "x86") | ("windows", "x86_64") => Some(Self::WindowsX86),
            ("windows", "arm") | ("windows", "aarch64") => Some(Self::WindowsArm64),
            _ => None,
        }
    }
}

pub fn _download_url_for_current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") | ("linux", "x86") => {
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz"
        }
        ("linux", "aarch64") | ("linux", "arm") => {
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linuxarm64-gpl.tar.xz"
        }
        ("windows", "x86_64") | ("windows", "x86") => {
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
        }
        ("windows", "aarch64") | ("windows", "arm") => {
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-winarm64-gpl.zip"
        }
        ("macos", "x86_64") | ("macos", "x86") => "https://evermeet.cx/ffmpeg/getrelease/zip",
        ("macos", "aarch64") | ("macos", "arm") => "https://evermeet.cx/ffmpeg/getrelease/arm64/zip",
        _ => "",
    }
}

pub fn app_bin_dir() -> PathBuf {
    match std::env::current_exe() {
        Ok(exe) => exe
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf())
            .join("bin"),
        Err(_) => Path::new(env!("CARGO_MANIFEST_DIR")).join("bin"),
    }
}

pub fn app_ffmpeg_dir() -> PathBuf {
    app_bin_dir().join("ffmpeg")
}

pub fn bundled_ffmpeg_dir() -> PathBuf {
    app_ffmpeg_dir()
}

fn path_lookup(binary_name: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;

    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }

        let windows_candidate = dir.join(format!("{binary_name}.exe"));
        if windows_candidate.is_file() {
            return Some(windows_candidate);
        }
    }

    None
}

fn resolve_binary(binary_name: &str) -> PathBuf {
    if let Some(candidate) = path_lookup(binary_name) {
        return candidate;
    }

    let base_dir = bundled_ffmpeg_dir();
    let target = match Platform::current().unwrap_or(Platform::Linux) {
        Platform::Linux => base_dir.join("linux").join(binary_name),
        Platform::Mac => base_dir.join("mac").join(binary_name),
        Platform::WindowsX86 => base_dir.join("windows").join("x86").join(format!("{binary_name}.exe")),
        Platform::WindowsArm64 => base_dir.join("windows").join("arm").join(format!("{binary_name}.exe")),
    };

    if target.exists() {
        target
    } else {
        PathBuf::from(binary_name)
    }
}

pub fn ffmpeg_binary_path() -> PathBuf {
    resolve_binary("ffmpeg")
}

pub fn ffprobe_binary_path() -> PathBuf {
    resolve_binary("ffprobe")
}

pub fn ffplay_binary_path() -> PathBuf {
    resolve_binary("ffplay")
}

pub fn ffmpeg_selection() -> std::collections::BTreeMap<&'static str, PathBuf> {
    let mut map = std::collections::BTreeMap::new();
    map.insert("ffmpeg", ffmpeg_binary_path());
    map.insert("ffprobe", ffprobe_binary_path());
    map.insert("ffplay", ffplay_binary_path());
    map
}

#[derive(Clone, Debug)]
pub struct FFFamily {
    pub ffprobe: PathBuf,
    pub ffmpeg: PathBuf,
    pub ffplay: PathBuf,
}

impl FFFamily {
    pub fn detect() -> Self {
        Self {
            ffprobe: ffprobe_binary_path(),
            ffmpeg: ffmpeg_binary_path(),
            ffplay: ffplay_binary_path(),
        }
    }

    pub fn status_for(&self, binary: &str) -> &'static str {
        let path = match binary {
            "ffprobe" => &self.ffprobe,
            "ffmpeg" => &self.ffmpeg,
            "ffplay" => &self.ffplay,
            _ => return "error",
        };

        if path.as_os_str().is_empty() || !path.is_file() {
            "error"
        } else {
            "active"
        }
    }

    pub fn is_available(&self) -> bool {
        // The editing/probing tools require ffmpeg + ffprobe.
        // ffplay is useful for playback, but it should not block the whole app.
        self.ffmpeg.is_file() && self.ffprobe.is_file()
    }

    pub fn _is_ffplay_available(&self) -> bool {
        self.ffplay.is_file()
    }

    pub fn _missing_message(&self) -> Option<String> {
        if self.is_available() {
            return None;
        }

        Some(
            "FFmpeg/ffprobe were not detected. Install FFmpeg and make sure ffmpeg and ffprobe are on PATH."
                .to_string(),
        )
    }

    pub fn _with_overrides(
        ffmpeg: Option<PathBuf>,
        ffprobe: Option<PathBuf>,
        ffplay: Option<PathBuf>,
    ) -> Self {
        let detected = Self::detect();

        Self {
            ffprobe: ffprobe.unwrap_or(detected.ffprobe),
            ffmpeg: ffmpeg.unwrap_or(detected.ffmpeg),
            ffplay: ffplay.unwrap_or(detected.ffplay),
        }
    }

    pub fn _reset(&mut self) {
        let detected = Self::detect();
        self.ffprobe = detected.ffprobe;
        self.ffmpeg = detected.ffmpeg;
        self.ffplay = detected.ffplay;
    }
}

fn command_for(binary_name: &str) -> Command {
    let path = resolve_binary(binary_name);
    Command::new(path)
}

pub fn ffmpeg_command() -> Command {
    command_for("ffmpeg")
}

pub fn ffprobe_command() -> Command {
    command_for("ffprobe")
}

pub fn _ffplay_command() -> Command {
    command_for("ffplay")
}

pub fn ffmpeg_with_args<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = ffmpeg_command();
    command.args(args);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_url_matches_platform() {
        let url = _download_url_for_current_platform();
        let expected = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz",
            ("linux", "aarch64") => "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
            ("windows", "x86_64") | ("windows", "x86") => "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip",
            ("windows", "aarch64") | ("windows", "arm") => "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-winarm64-gpl.zip",
            ("macos", "x86_64") => "https://evermeet.cx/ffmpeg/getrelease/zip",
            ("macos", "aarch64") => "https://evermeet.cx/ffmpeg/getrelease/arm64/zip",
            _ => "",
        };

        assert_eq!(url, expected);
    }

    #[test]
    fn detects_supported_platform_binary() {
        assert!(!ffmpeg_binary_path().as_os_str().is_empty());
        assert!(!ffprobe_binary_path().as_os_str().is_empty());
        assert!(!ffplay_binary_path().as_os_str().is_empty());
    }

    #[test]
    fn command_builder_accepts_custom_args() {
        let mut command = ffmpeg_command();
        command.args(["-hide_banner", "-version"]);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert!(args.iter().any(|arg| arg == "-hide_banner"));
        assert!(args.iter().any(|arg| arg == "-version"));
    }
}
