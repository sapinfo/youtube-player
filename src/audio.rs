use std::path::{Path, PathBuf};
use std::process::Command;

use crate::player::{augmented_path, EXTRA_PATHS};

/// 주어진 YouTube URL과 출력 폴더로 yt-dlp에 넘길 인자 목록을 만든다.
/// mp3로 오디오만 추출하고 최고 품질로 저장한다.
pub fn build_extract_args(url: &str, out_dir: &Path) -> Vec<String> {
    let output_template = out_dir.join("%(title)s.%(ext)s");
    vec![
        "-x".to_string(),
        "--audio-format".to_string(),
        "mp3".to_string(),
        "--audio-quality".to_string(),
        "0".to_string(),
        "-o".to_string(),
        output_template.to_string_lossy().to_string(),
        url.to_string(),
    ]
}

/// yt-dlp 실행 파일을 찾는다: 알려진 Homebrew 경로 우선, 없으면 PATH로 해석되는 bare name.
fn yt_dlp_command() -> PathBuf {
    for dir in EXTRA_PATHS {
        let candidate = PathBuf::from(dir).join("yt-dlp");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("yt-dlp")
}

/// ffmpeg가 PATH(보강된)에서 실행 가능한지 확인한다. mp3 변환에 필요하다.
pub fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .env("PATH", augmented_path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// yt-dlp로 오디오를 mp3로 추출한다. 종료까지 대기하므로 백그라운드 스레드에서 호출할 것.
/// 성공 시 저장 폴더 경로 문자열을 돌려준다.
pub fn extract_mp3(url: &str, out_dir: &Path) -> Result<String, String> {
    let args = build_extract_args(url, out_dir);
    let output = Command::new(yt_dlp_command())
        .args(&args)
        .env("PATH", augmented_path())
        .output()
        .map_err(|e| {
            format!("Failed to launch yt-dlp: {e}. yt-dlp and ffmpeg must be installed.")
        })?;
    if output.status.success() {
        Ok(out_dir.to_string_lossy().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim().lines().last().unwrap_or("unknown error");
        Err(format!("Extraction failed: {detail}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_mp3_extraction_args_in_order() {
        let args = build_extract_args("URL", Path::new("/tmp/out"));
        assert_eq!(
            args,
            vec![
                "-x".to_string(),
                "--audio-format".to_string(),
                "mp3".to_string(),
                "--audio-quality".to_string(),
                "0".to_string(),
                "-o".to_string(),
                "/tmp/out/%(title)s.%(ext)s".to_string(),
                "URL".to_string(),
            ]
        );
    }

    #[test]
    fn url_is_last_argument() {
        let args = build_extract_args("https://youtu.be/abc", Path::new("/tmp"));
        assert_eq!(args.last().unwrap(), "https://youtu.be/abc");
    }
}
