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
