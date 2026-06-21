/// YouTube URL인지 간단히 검증한다(youtube.com 또는 youtu.be 호스트 포함, http로 시작).
pub fn validate(url: &str) -> bool {
    let lowered = url.trim().to_ascii_lowercase();
    if !(lowered.starts_with("http://") || lowered.starts_with("https://")) {
        return false;
    }
    lowered.contains("youtube.com/") || lowered.contains("youtu.be/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_standard_watch_url() {
        assert!(validate("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn accepts_short_url() {
        assert!(validate("https://youtu.be/dQw4w9WgXcQ"));
    }

    #[test]
    fn rejects_empty() {
        assert!(!validate(""));
    }

    #[test]
    fn rejects_non_youtube() {
        assert!(!validate("https://example.com/watch?v=abc"));
    }

    #[test]
    fn rejects_missing_scheme() {
        assert!(!validate("www.youtube.com/watch?v=abc"));
    }
}
