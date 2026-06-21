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
}
