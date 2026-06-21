# youtube-player Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** macOS GUI 앱으로 YouTube URL을 이름과 함께 저장/수정/삭제/순서이동하고, 골라서 mpv 작은 별도 창(항상 위 옵션, 크기 프리셋)으로 재생한다.

**Architecture:** eframe/egui 단일 바이너리. 순수 로직(저장소 CRUD, URL 검증, mpv 인자 구성)을 GUI에서 분리한 모듈로 두고 단위 테스트한다. 상태(`AppData`)는 OS 설정 디렉터리의 단일 JSON 파일에 영속화한다. 재생은 `std::process::Command`로 mpv 자식 프로세스를 띄우고, mpv가 내부적으로 yt-dlp를 호출한다.

**Tech Stack:** Rust 2021, eframe/egui 0.29, serde + serde_json, directories 5, std::process::Command, mpv + yt-dlp(런타임 외부 의존).

---

## File Structure

- `Cargo.toml` — 패키지 메타데이터 + 의존성
- `src/main.rs` — eframe 진입점, 윈도우 생성, `App` 실행
- `src/url.rs` — `validate(url) -> bool` YouTube URL 검증 (순수 함수, 테스트)
- `src/player.rs` — `WindowSize`, `Settings`, `build_args`, `is_mpv_available`, `play` (인자 구성은 순수 함수로 분리, 테스트)
- `src/store.rs` — `Entry`, `AppData`, JSON load/save, CRUD/이동 (순수 함수, 테스트)
- `src/app.rs` — egui UI 상태 + 렌더링 (`eframe::App` 구현, 수동 검증)

모듈 간 의존: `app` → (`store`, `player`, `url`). `store`는 `player::{Settings, WindowSize}`를 재사용해 `AppData`에 담는다.

---

## Task 1: 프로젝트 스캐폴딩

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Cargo.toml 작성**

`Cargo.toml`:

```toml
[package]
name = "youtube-player"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.29"
egui = "0.29"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
directories = "5"
```

- [ ] **Step 2: 최소 main.rs 작성 (빈 창)**

`src/main.rs`:

```rust
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "YouTube Player",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

#[derive(Default)]
struct App;

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("YouTube Player");
        });
    }
}
```

- [ ] **Step 3: 빌드 확인**

Run: `cargo build`
Expected: 의존성 다운로드 후 컴파일 성공(`Finished`). 경고는 허용.

> 참고: eframe 0.29 API 기준(`update`/`CentralPanel::default().show(ctx, ...)`). 만약 설치된 버전에서 컴파일 에러가 나면 `cargo update` 대신 `Cargo.toml`의 `eframe`/`egui` 버전을 동일 마이너로 고정하고, `eframe::run_native`의 클로저 반환이 `Ok(...)`(Result) 형태인지 확인한다.

- [ ] **Step 4: 커밋**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: scaffold eframe app with empty window"
```

---

## Task 2: URL 검증 모듈 (`url.rs`)

**Files:**
- Create: `src/url.rs`
- Modify: `src/main.rs` (모듈 선언 추가)

- [ ] **Step 1: 실패하는 테스트 작성**

`src/url.rs`:

```rust
/// YouTube URL인지 간단히 검증한다(youtube.com 또는 youtu.be 호스트 포함, http로 시작).
pub fn validate(url: &str) -> bool {
    unimplemented!()
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
```

`src/main.rs` 상단에 모듈 선언 추가 (`fn main` 위):

```rust
mod url;
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --bin youtube-player url::`
Expected: 컴파일은 되지만 `unimplemented!()`로 패닉하여 테스트 FAIL.

- [ ] **Step 3: 최소 구현 작성**

`src/url.rs`의 `validate` 본문을 교체:

```rust
pub fn validate(url: &str) -> bool {
    let lowered = url.trim().to_ascii_lowercase();
    if !(lowered.starts_with("http://") || lowered.starts_with("https://")) {
        return false;
    }
    lowered.contains("youtube.com/") || lowered.contains("youtu.be/")
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `cargo test --bin youtube-player url::`
Expected: 5개 테스트 모두 PASS.

- [ ] **Step 5: 커밋**

```bash
git add src/url.rs src/main.rs
git commit -m "feat: add YouTube URL validation"
```

---

## Task 3: mpv 인자 구성과 설정 타입 (`player.rs`)

**Files:**
- Create: `src/player.rs`
- Modify: `src/main.rs` (모듈 선언 추가)

- [ ] **Step 1: 실패하는 테스트 작성**

`src/player.rs`:

```rust
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
    unimplemented!()
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
```

`src/main.rs`에 모듈 선언 추가:

```rust
mod player;
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --bin youtube-player player::`
Expected: `unimplemented!()` 패닉으로 FAIL.

- [ ] **Step 3: 최소 구현 작성**

`src/player.rs`의 `build_args` 본문을 교체:

```rust
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
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `cargo test --bin youtube-player player::`
Expected: 3개 테스트 PASS.

- [ ] **Step 5: 커밋**

```bash
git add src/player.rs src/main.rs
git commit -m "feat: add player settings and mpv arg builder"
```

---

## Task 4: mpv 실행과 가용성 점검 (`player.rs`에 추가)

**Files:**
- Modify: `src/player.rs`

`is_mpv_available`와 `play`는 실제 프로세스/시스템 상태에 의존하므로 자동 테스트 대신 컴파일과 수동 검증으로 확인한다.

- [ ] **Step 1: 함수 추가**

`src/player.rs`의 `build_args` 정의 아래(테스트 모듈 위)에 추가:

```rust
use std::process::Command;

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
```

- [ ] **Step 2: 빌드 확인**

Run: `cargo build`
Expected: 컴파일 성공. (미사용 경고가 있을 수 있으나 Task 6에서 app이 호출하면 사라진다.)

- [ ] **Step 3: 커밋**

```bash
git add src/player.rs
git commit -m "feat: add mpv availability check and spawn"
```

---

## Task 5: 저장소 — 데이터 모델과 CRUD (`store.rs`)

**Files:**
- Create: `src/store.rs`
- Modify: `src/main.rs` (모듈 선언 추가)

- [ ] **Step 1: 실패하는 테스트 작성**

`src/store.rs`:

```rust
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
```

`src/main.rs`에 모듈 선언 추가:

```rust
mod store;
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --bin youtube-player store::`
Expected: 처음에는 컴파일/실행되어 모두 PASS할 수도 있다(로직이 인라인 구현됨). 만약 PASS하면 이 태스크의 TDD 목적상 의도된 것이므로 그대로 Step 5로 진행한다. 컴파일 에러(예: `crate::player::Settings` 경로)면 `player` 모듈 선언이 `store`보다 먼저 있는지 확인한다.

- [ ] **Step 3: (해당 없음 — 구현이 Step 1에 포함됨)**

CRUD 로직은 단순하여 Step 1에 함께 작성했다. 별도 구현 없음.

- [ ] **Step 4: 테스트 통과 확인**

Run: `cargo test --bin youtube-player store::`
Expected: 6개 테스트 PASS.

- [ ] **Step 5: 커밋**

```bash
git add src/store.rs src/main.rs
git commit -m "feat: add AppData model with CRUD and reordering"
```

---

## Task 6: 저장소 — JSON 영속화 (`store.rs`에 추가)

**Files:**
- Modify: `src/store.rs`

- [ ] **Step 1: 실패하는 테스트 작성**

`src/store.rs`의 `impl AppData` 블록 아래, `#[cfg(test)]` 위에 추가:

```rust
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
```

`#[cfg(test)] mod tests`의 `use super::*;` 아래에 라운드트립 테스트 추가:

```rust
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
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --bin youtube-player store::`
Expected: 새 테스트가 추가되어 컴파일·실행. 구현이 함께 작성됐으므로 PASS할 수 있다. 컴파일 에러가 나면 메시지대로 수정.

- [ ] **Step 3: (해당 없음 — 구현이 Step 1에 포함됨)**

- [ ] **Step 4: 전체 테스트 통과 확인**

Run: `cargo test`
Expected: url/player/store 모든 테스트 PASS.

- [ ] **Step 5: 커밋**

```bash
git add src/store.rs
git commit -m "feat: add JSON persistence for AppData"
```

---

## Task 7: GUI 조립 (`app.rs`)

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs` (모듈 선언 + `App` 교체)

GUI는 자동 테스트 대상이 아니므로 컴파일 + 수동 검증으로 확인한다.

- [ ] **Step 1: app.rs 작성**

`src/app.rs`:

```rust
use eframe::egui;

use crate::player::{self, Settings, WindowSize};
use crate::store::{self, AppData};
use crate::url;

/// 추가/수정 폼의 현재 입력 상태.
#[derive(Default)]
struct Form {
    name: String,
    url: String,
    editing: Option<usize>, // Some(i)면 i번 항목 수정 중, None이면 새 항목 추가
}

pub struct App {
    data: AppData,
    form: Form,
    mpv_available: bool,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: store::load(),
            form: Form::default(),
            mpv_available: player::is_mpv_available(),
            status: String::new(),
        }
    }
}

impl App {
    fn persist(&mut self) {
        if let Err(e) = store::save(&self.data) {
            self.status = e;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("YouTube Player");

            if !self.mpv_available {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 60, 60),
                    "mpv가 필요합니다: 터미널에서 `brew install mpv` 실행 후 앱을 다시 시작하세요.",
                );
            }

            ui.separator();

            // --- 재생 설정 ---
            ui.label("재생 설정");
            let mut changed = false;
            changed |= ui.checkbox(&mut self.data.settings.ontop, "항상 위(ontop)").changed();
            egui::ComboBox::from_label("창 크기")
                .selected_text(self.data.settings.window_size.label())
                .show_ui(ui, |ui| {
                    for size in [WindowSize::Small, WindowSize::Medium, WindowSize::Large] {
                        changed |= ui
                            .selectable_value(&mut self.data.settings.window_size, size, size.label())
                            .changed();
                    }
                });
            if changed {
                self.persist();
            }

            ui.separator();

            // --- 저장된 목록 ---
            ui.label("저장된 영상");
            let mut action: Option<ListAction> = None;
            let count = self.data.entries.len();
            for (i, entry) in self.data.entries.iter().enumerate() {
                ui.horizontal(|ui| {
                    let play_btn = ui.add_enabled(self.mpv_available, egui::Button::new("▶"));
                    if play_btn.clicked() {
                        action = Some(ListAction::Play(i));
                    }
                    if ui.button("✎").clicked() {
                        action = Some(ListAction::Edit(i));
                    }
                    if ui.button("🗑").clicked() {
                        action = Some(ListAction::Delete(i));
                    }
                    if ui.add_enabled(i > 0, egui::Button::new("↑")).clicked() {
                        action = Some(ListAction::Up(i));
                    }
                    if ui.add_enabled(i + 1 < count, egui::Button::new("↓")).clicked() {
                        action = Some(ListAction::Down(i));
                    }
                    ui.label(if entry.name.is_empty() { &entry.url } else { &entry.name });
                });
            }

            if let Some(a) = action {
                self.handle_list_action(a);
            }

            ui.separator();

            // --- 추가/수정 폼 ---
            ui.label(if self.form.editing.is_some() { "수정" } else { "추가" });
            ui.horizontal(|ui| {
                ui.label("이름");
                ui.text_edit_singleline(&mut self.form.name);
            });
            ui.horizontal(|ui| {
                ui.label("URL");
                ui.text_edit_singleline(&mut self.form.url);
            });
            ui.horizontal(|ui| {
                let label = if self.form.editing.is_some() { "저장" } else { "추가" };
                if ui.button(label).clicked() {
                    self.submit_form();
                }
                if self.form.editing.is_some() && ui.button("취소").clicked() {
                    self.form = Form::default();
                }
            });

            if !self.status.is_empty() {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(200, 60, 60), &self.status);
            }
        });
    }
}

enum ListAction {
    Play(usize),
    Edit(usize),
    Delete(usize),
    Up(usize),
    Down(usize),
}

impl App {
    fn handle_list_action(&mut self, action: ListAction) {
        match action {
            ListAction::Play(i) => {
                if let Some(e) = self.data.entries.get(i) {
                    if !url::validate(&e.url) {
                        self.status = "잘못된 YouTube URL입니다.".into();
                    } else {
                        let url = e.url.clone();
                        let settings = self.data.settings.clone();
                        match player::play(&url, &settings) {
                            Ok(()) => self.status = "재생 중…".into(),
                            Err(e) => self.status = e,
                        }
                    }
                }
            }
            ListAction::Edit(i) => {
                if let Some(e) = self.data.entries.get(i) {
                    self.form = Form { name: e.name.clone(), url: e.url.clone(), editing: Some(i) };
                }
            }
            ListAction::Delete(i) => {
                self.data.delete(i);
                if self.form.editing == Some(i) {
                    self.form = Form::default();
                }
                self.persist();
            }
            ListAction::Up(i) => {
                self.data.move_up(i);
                self.persist();
            }
            ListAction::Down(i) => {
                self.data.move_down(i);
                self.persist();
            }
        }
    }

    fn submit_form(&mut self) {
        let name = self.form.name.trim().to_string();
        let url_value = self.form.url.trim().to_string();
        if url_value.is_empty() || !url::validate(&url_value) {
            self.status = "잘못된 YouTube URL입니다.".into();
            return;
        }
        match self.form.editing {
            Some(i) => self.data.update(i, name, url_value),
            None => self.data.add(name, url_value),
        }
        self.form = Form::default();
        self.status.clear();
        self.persist();
    }
}
```

- [ ] **Step 2: main.rs를 app.rs 사용하도록 교체**

`src/main.rs` 전체를 교체:

```rust
mod app;
mod player;
mod store;
mod url;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "YouTube Player",
        options,
        Box::new(|_cc| Ok(Box::<app::App>::default())),
    )
}
```

- [ ] **Step 3: 전체 빌드 + 테스트**

Run: `cargo build && cargo test`
Expected: 빌드 성공, url/player/store 테스트 전부 PASS.

> egui 0.29에서 `add_enabled`, `Button::new`, `ComboBox::from_label`, `selectable_value`, `text_edit_singleline`, `colored_label`, `checkbox`는 모두 존재한다. 시그니처 불일치 컴파일 에러가 나면 해당 위젯의 `cargo doc --open` 시그니처에 맞춰 호출만 조정한다(로직은 그대로).

- [ ] **Step 4: 커밋**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: build egui UI with list CRUD, reorder, settings, and playback"
```

---

## Task 8: 수동 통합 검증 + README

**Files:**
- Create: `README.md`

- [ ] **Step 1: 앱 실행 수동 검증**

Run: `cargo run`
검증 항목:
1. 창이 뜨고 "YouTube Player" 제목과 재생 설정/목록/추가 폼이 보인다.
2. 이름 "테스트", URL `https://youtu.be/dQw4w9WgXcQ` 입력 후 [추가] → 목록에 "테스트" 행 생김.
3. ▶ 클릭 → mpv가 작은 별도 창(640x360 근처)으로 영상 재생.
4. "항상 위" 체크 후 다시 ▶ → mpv 창이 다른 창 위에 뜸.
5. 창 크기 프리셋을 "작음"으로 바꾸고 ▶ → 더 작은 창.
6. ✎로 수정, ↑/↓로 순서 이동, 🗑로 삭제 동작 확인.
7. 앱 종료 후 `cargo run` 재실행 → 목록과 설정이 유지됨(JSON 저장 확인).
8. 잘못된 URL(예: `https://example.com`) 추가 시 빨간 안내 표시.

- [ ] **Step 2: README 작성**

`README.md`:

```markdown
# YouTube Player

YouTube URL을 이름과 함께 저장/관리하고, 골라서 작은 mpv 창으로 재생하는 macOS GUI 앱.

## 사전 요구사항

- [Rust](https://rustup.rs) (2021 edition)
- mpv: `brew install mpv`
- yt-dlp: `brew install yt-dlp` (mpv가 YouTube를 해석할 때 내부적으로 사용)

## 실행

```bash
cargo run
```

## 사용법

1. 하단 폼에 이름과 YouTube URL을 입력하고 **추가**.
2. 목록에서 항목의 **▶**를 눌러 재생. 영상은 작은 별도 창으로 뜬다.
3. **✎** 수정, **🗑** 삭제, **↑/↓** 순서 이동.
4. 상단 **재생 설정**에서 "항상 위(ontop)"와 창 크기 프리셋을 조절. 설정과 목록은 자동 저장된다.

## 데이터 저장 위치

`~/Library/Application Support/youtube-player/data.json` (macOS)

## mpv 창 크기 조절

재생 중 mpv 창 모서리를 드래그하거나, `Alt+0`(절반) / `Alt+1`(원래) / `Alt+2`(두 배) 단축키 사용.
```

- [ ] **Step 3: 커밋**

```bash
git add README.md
git commit -m "docs: add README with setup and usage"
```

---

## Self-Review 결과

- **Spec coverage:** 데이터 모델(Task 5), JSON 영속화(Task 6), CRUD+순서이동(Task 5), mpv 작은 창/ontop/크기 프리셋(Task 3·4), URL 검증(Task 2), mpv 미설치 배너 + 재생 비활성화(Task 7), 에러 처리(Task 7), 테스트(Task 2·3·5·6) — 스펙 전 항목이 태스크에 매핑됨.
- **Placeholder scan:** 코드가 필요한 모든 스텝에 실제 코드 포함. "구현이 Step 1에 포함됨" 스텝은 단순 CRUD/직렬화라 TDD 형식상 분리하지 않고 명시.
- **Type consistency:** `AppData`, `Entry`, `Settings`, `WindowSize`, `build_args`, `play`, `is_mpv_available`, `load/save`, `validate` 시그니처가 Task 간 일치. `store`가 `player::Settings`를 재사용하므로 main.rs에서 `player` 모듈이 먼저 선언되도록 Task 7에서 정리됨.
