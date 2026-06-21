use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::player::Settings;

/// 저장되는 한 항목: 이름표 + URL.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub name: String,
    pub url: String,
}

/// 디스크에 직렬화되는 전체 앱 상태.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppData {
    pub entries: Vec<Entry>,
    pub settings: Settings,
}

impl AppData {
    pub fn add(&mut self, name: String, url: String) {
        self.entries.push(Entry { name, url });
    }

    pub fn update(&mut self, index: usize, name: String, url: String) {
        if let Some(e) = self.entries.get_mut(index) {
            e.name = name;
            e.url = url;
        }
    }

    pub fn delete(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }

    pub fn move_up(&mut self, index: usize) {
        if index > 0 && index < self.entries.len() {
            self.entries.swap(index, index - 1);
        }
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.entries.len() {
            self.entries.swap(index, index + 1);
        }
    }
}

/// 데이터 파일 경로: OS 데이터 디렉터리/youtube-player/data.json
pub fn data_path() -> PathBuf {
    use directories::ProjectDirs;
    if let Some(dirs) = ProjectDirs::from("com", "nuwavenow", "youtube-player") {
        dirs.data_dir().join("data.json")
    } else {
        PathBuf::from("data.json")
    }
}

/// 지정 경로에서 AppData를 읽는다. 파일이 없거나 깨졌으면 기본값을 반환한다.
pub fn load_from(path: &std::path::Path) -> AppData {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => AppData::default(),
    }
}

/// 지정 경로에 AppData를 저장한다. 상위 디렉터리는 필요 시 생성한다.
pub fn save_to(path: &std::path::Path, data: &AppData) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("디렉터리 생성 실패: {e}"))?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| format!("직렬화 실패: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("저장 실패: {e}"))
}

/// 표준 경로에서 로드.
pub fn load() -> AppData {
    load_from(&data_path())
}

/// 표준 경로로 저장.
pub fn save(data: &AppData) -> Result<(), String> {
    save_to(&data_path(), data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_appends_entry() {
        let mut d = AppData::default();
        d.add("오프닝".into(), "https://youtu.be/a".into());
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].name, "오프닝");
    }

    #[test]
    fn update_changes_fields() {
        let mut d = AppData::default();
        d.add("a".into(), "u1".into());
        d.update(0, "b".into(), "u2".into());
        assert_eq!(d.entries[0], Entry { name: "b".into(), url: "u2".into() });
    }

    #[test]
    fn delete_removes_entry() {
        let mut d = AppData::default();
        d.add("a".into(), "u".into());
        d.delete(0);
        assert!(d.entries.is_empty());
    }

    #[test]
    fn move_up_swaps_with_previous() {
        let mut d = AppData::default();
        d.add("a".into(), "u".into());
        d.add("b".into(), "u".into());
        d.move_up(1);
        assert_eq!(d.entries[0].name, "b");
    }

    #[test]
    fn move_up_at_top_is_noop() {
        let mut d = AppData::default();
        d.add("a".into(), "u".into());
        d.move_up(0);
        assert_eq!(d.entries[0].name, "a");
    }

    #[test]
    fn move_down_at_bottom_is_noop() {
        let mut d = AppData::default();
        d.add("a".into(), "u".into());
        d.move_down(0);
        assert_eq!(d.entries[0].name, "a");
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ytp_test_{}", std::process::id()));
        let path = dir.join("data.json");
        let mut d = AppData::default();
        d.add("오프닝".into(), "https://youtu.be/a".into());
        d.settings.ontop = true;

        save_to(&path, &d).expect("save");
        let loaded = load_from(&path);

        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].name, "오프닝");
        assert!(loaded.settings.ontop);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let path = std::env::temp_dir().join("ytp_does_not_exist_xyz/data.json");
        let loaded = load_from(&path);
        assert!(loaded.entries.is_empty());
    }
}
