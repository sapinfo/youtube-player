use std::process::Command;

use serde::{Deserialize, Serialize};

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
            WindowSize::Small => "작음 (480x270)",
            WindowSize::Medium => "중간 (640x360)",
            WindowSize::Large => "큼 (960x540)",
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
    Command::new("mpv")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 설정에 따라 mpv 자식 프로세스를 띄운다. 즉시 반환한다(재생 종료를 기다리지 않음).
pub fn play(url: &str, settings: &Settings) -> Result<(), String> {
    let args = build_args(url, settings);
    Command::new("mpv")
        .args(&args)
        .spawn()
        .map(|_child| ())
        .map_err(|e| format!("mpv 실행 실패: {e}. mpv와 yt-dlp가 모두 설치돼 있어야 합니다."))
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
