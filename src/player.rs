use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Directories to search for mpv and to expose to the spawned mpv process,
/// in addition to the inherited PATH. A `.app` launched from Finder gets a
/// minimal PATH that excludes Homebrew locations, so mpv (and the yt-dlp it
/// calls internally) would otherwise not be found.
pub(crate) const EXTRA_PATHS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];

/// Resolve the mpv executable: prefer known Homebrew locations, otherwise fall
/// back to the bare name (resolved via the inherited PATH).
fn mpv_command() -> PathBuf {
    for dir in EXTRA_PATHS {
        let candidate = PathBuf::from(dir).join("mpv");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("mpv")
}

/// PATH that prepends the extra directories so mpv can locate yt-dlp.
pub(crate) fn augmented_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = EXTRA_PATHS.iter().map(|s| s.to_string()).collect();
    if !current.is_empty() {
        parts.push(current);
    }
    parts.join(":")
}

/// mpv 시작 창 크기 프리셋.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowSize {
    Small,
    Medium,
    Large,
}

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize::Medium
    }
}

impl WindowSize {
    /// `--autofit`에 쓸 "WxH" 문자열.
    pub fn dimensions(self) -> &'static str {
        match self {
            WindowSize::Small => "480x270",
            WindowSize::Medium => "640x360",
            WindowSize::Large => "960x540",
        }
    }

    /// UI 드롭다운 라벨.
    pub fn label(self) -> &'static str {
        match self {
            WindowSize::Small => "Small (480x270)",
            WindowSize::Medium => "Medium (640x360)",
            WindowSize::Large => "Large (960x540)",
        }
    }
}

/// 재생 설정.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub ontop: bool,
    pub window_size: WindowSize,
}

/// 주어진 URL과 설정으로 mpv에 넘길 인자 목록을 만든다.
pub fn build_args(url: &str, settings: &Settings) -> Vec<String> {
    let mut args = vec![
        "--no-fullscreen".to_string(),
        format!("--autofit={}", settings.window_size.dimensions()),
    ];
    if settings.ontop {
        args.push("--ontop".to_string());
    }
    args.push(url.to_string());
    args
}

/// mpv 실행 파일이 PATH에 있는지 확인한다.
pub fn is_mpv_available() -> bool {
    Command::new(mpv_command())
        .arg("--version")
        .env("PATH", augmented_path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 설정에 따라 mpv 자식 프로세스를 띄운다. 즉시 반환한다(재생 종료를 기다리지 않음).
pub fn play(url: &str, settings: &Settings) -> Result<(), String> {
    let args = build_args(url, settings);
    Command::new(mpv_command())
        .args(&args)
        .env("PATH", augmented_path())
        .spawn()
        .map(|_child| ())
        .map_err(|e| format!("Failed to launch mpv: {e}. Both mpv and yt-dlp must be installed."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_windowed_with_autofit() {
        let args = build_args("URL", &Settings { ontop: false, window_size: WindowSize::Medium });
        assert_eq!(
            args,
            vec![
                "--no-fullscreen".to_string(),
                "--autofit=640x360".to_string(),
                "URL".to_string(),
            ]
        );
    }

    #[test]
    fn adds_ontop_when_enabled() {
        let args = build_args("URL", &Settings { ontop: true, window_size: WindowSize::Small });
        assert_eq!(
            args,
            vec![
                "--no-fullscreen".to_string(),
                "--autofit=480x270".to_string(),
                "--ontop".to_string(),
                "URL".to_string(),
            ]
        );
    }

    #[test]
    fn large_size_dimensions() {
        let args = build_args("URL", &Settings { ontop: false, window_size: WindowSize::Large });
        assert!(args.contains(&"--autofit=960x540".to_string()));
    }
}
