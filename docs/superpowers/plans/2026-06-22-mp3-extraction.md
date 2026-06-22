# MP3 Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a per-entry "MP3" button that extracts audio from a saved YouTube URL to an mp3 file in a user-chosen folder, running in the background with status feedback.

**Architecture:** A new `audio.rs` module owns mp3 extraction (yt-dlp downloads + ffmpeg converts), reusing player.rs's PATH-resolution helpers (exposed as `pub(crate)`). The UI spawns a background thread per extraction and receives the result over an `mpsc` channel, polling it each frame while a repaint is requested.

**Tech Stack:** Rust, eframe/egui 0.29, rfd (folder picker), std::thread + std::sync::mpsc, external `yt-dlp` and `ffmpeg` binaries.

---

## File Structure

- **Modify `src/player.rs`** — change `EXTRA_PATHS` and `augmented_path()` from private to `pub(crate)` so audio.rs reuses them. No behavior change.
- **Create `src/audio.rs`** — mp3 extraction: locate yt-dlp, check ffmpeg, build args (pure, tested), run extraction (blocking, called from a thread).
- **Modify `src/main.rs`** — register `mod audio;`.
- **Modify `src/app.rs`** — add `ffmpeg_available`, `extracting`, `extract_rx` fields; the MP3 button; channel polling in `update()`; the ffmpeg banner.

---

## Task 1: Expose player.rs PATH helpers for reuse

**Files:**
- Modify: `src/player.rs:10` (`EXTRA_PATHS`), `src/player.rs:25` (`augmented_path`)

- [ ] **Step 1: Change `EXTRA_PATHS` visibility**

In `src/player.rs`, change the constant declaration from:

```rust
const EXTRA_PATHS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];
```

to:

```rust
pub(crate) const EXTRA_PATHS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];
```

- [ ] **Step 2: Change `augmented_path` visibility**

In `src/player.rs`, change the function signature from:

```rust
fn augmented_path() -> String {
```

to:

```rust
pub(crate) fn augmented_path() -> String {
```

- [ ] **Step 3: Verify it still compiles and tests pass**

Run: `cargo test`
Expected: PASS (existing player tests still pass; no behavior change).

- [ ] **Step 4: Commit**

```bash
git add src/player.rs
git commit -m "refactor: expose player PATH helpers as pub(crate) for reuse"
```

---

## Task 2: `build_extract_args` pure function (TDD)

**Files:**
- Create: `src/audio.rs`
- Modify: `src/main.rs:1-4` (add `mod audio;`)

- [ ] **Step 1: Create `src/audio.rs` with the failing test**

Create `src/audio.rs` with exactly this content:

```rust
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
```

- [ ] **Step 2: Register the module in main.rs**

In `src/main.rs`, the module declarations are:

```rust
mod app;
mod player;
mod store;
mod url;
```

Change them to (alphabetical, `audio` first):

```rust
mod app;
mod audio;
mod player;
mod store;
mod url;
```

- [ ] **Step 3: Run the tests**

Run: `cargo test build_extract_args`
Expected: PASS for `builds_mp3_extraction_args_in_order` and `url_is_last_argument`.

> Note: `EXTRA_PATHS` and `Command`/`PathBuf` are imported but unused until Task 3; the compiler will warn (not error). That is expected and resolved in Task 3.

- [ ] **Step 4: Commit**

```bash
git add src/audio.rs src/main.rs
git commit -m "feat: add build_extract_args for yt-dlp mp3 extraction"
```

---

## Task 3: yt-dlp/ffmpeg resolution and `extract_mp3`

**Files:**
- Modify: `src/audio.rs`

- [ ] **Step 1: Add binary-resolution and extraction functions**

In `src/audio.rs`, insert the following functions between `build_extract_args` and the `#[cfg(test)] mod tests` block:

```rust
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
```

- [ ] **Step 2: Verify it compiles and existing tests still pass**

Run: `cargo test`
Expected: PASS. The earlier unused-import warnings from Task 2 are now resolved (`Command`, `PathBuf`, `EXTRA_PATHS` are all used).

- [ ] **Step 3: Commit**

```bash
git add src/audio.rs
git commit -m "feat: add yt-dlp/ffmpeg resolution and extract_mp3"
```

---

## Task 4: App state for extraction

**Files:**
- Modify: `src/app.rs:1-39` (imports, `App` struct, `Default` impl)

- [ ] **Step 1: Add the mpsc import**

In `src/app.rs`, the top of the file is:

```rust
use eframe::egui;

use crate::player::{self, WindowSize};
use crate::store::{self, AppData};
use crate::url;
```

Change it to:

```rust
use std::sync::mpsc::Receiver;

use eframe::egui;

use crate::audio;
use crate::player::{self, WindowSize};
use crate::store::{self, AppData};
use crate::url;
```

- [ ] **Step 2: Add fields to the `App` struct**

The current struct is:

```rust
pub struct App {
    data: AppData,
    form: Form,
    mpv_available: bool,
    status: String,
}
```

Change it to:

```rust
pub struct App {
    data: AppData,
    form: Form,
    mpv_available: bool,
    ffmpeg_available: bool,
    status: String,
    extracting: bool,
    extract_rx: Option<Receiver<Result<String, String>>>,
}
```

- [ ] **Step 3: Initialize the new fields in `Default`**

The current `Default` impl body is:

```rust
        Self {
            data: store::load(),
            form: Form::default(),
            mpv_available: player::is_mpv_available(),
            status: String::new(),
        }
```

Change it to:

```rust
        Self {
            data: store::load(),
            form: Form::default(),
            mpv_available: player::is_mpv_available(),
            ffmpeg_available: audio::is_ffmpeg_available(),
            status: String::new(),
            extracting: false,
            extract_rx: None,
        }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build`
Expected: builds (warnings about unused `ffmpeg_available`/`extracting`/`extract_rx` are fine until Task 5/6).

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: add extraction state fields to App"
```

---

## Task 5: Extraction trigger and channel polling

**Files:**
- Modify: `src/app.rs` — `ListAction` enum, `handle_list_action`, new `extract_to_mp3` method, `update()` polling.

- [ ] **Step 1: Add an `Extract` variant to `ListAction`**

The current enum is:

```rust
enum ListAction {
    Play(usize),
    Edit(usize),
    Delete(usize),
    Up(usize),
    Down(usize),
}
```

Change it to:

```rust
enum ListAction {
    Play(usize),
    Extract(usize),
    Edit(usize),
    Delete(usize),
    Up(usize),
    Down(usize),
}
```

- [ ] **Step 2: Handle the `Extract` action**

In `handle_list_action`, the `ListAction::Play(i) => { ... }` arm ends at the `}` before `ListAction::Edit(i)`. Immediately after the Play arm's closing `}`, add this new arm:

```rust
            ListAction::Extract(i) => {
                if let Some(e) = self.data.entries.get(i) {
                    if !url::validate(&e.url) {
                        self.status = "Invalid YouTube URL.".into();
                    } else {
                        let url = e.url.clone();
                        self.extract_to_mp3(url);
                    }
                }
            }
```

- [ ] **Step 3: Add the `extract_to_mp3` method**

In the same `impl App` block that contains `handle_list_action` and `play_local_file`, add this method (e.g. right after `play_local_file`):

```rust
    /// 폴더를 고른 뒤 백그라운드 스레드에서 mp3로 추출한다. 결과는 채널로 전달된다.
    fn extract_to_mp3(&mut self, url: String) {
        let Some(dir) = rfd::FileDialog::new()
            .set_title("Select a folder to save the MP3")
            .pick_folder()
        else {
            return; // 사용자가 취소함
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.extract_rx = Some(rx);
        self.extracting = true;
        self.status = "Extracting…".into();
        std::thread::spawn(move || {
            let result = audio::extract_mp3(&url, &dir);
            let _ = tx.send(result);
        });
    }
```

- [ ] **Step 4: Poll the channel at the top of `update()`**

The `update` method currently begins:

```rust
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
```

Insert the polling block between those two lines, so it reads:

```rust
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.extracting {
            if let Some(rx) = &self.extract_rx {
                match rx.try_recv() {
                    Ok(Ok(dir)) => {
                        self.status = format!("Saved to: {dir}");
                        self.extracting = false;
                        self.extract_rx = None;
                    }
                    Ok(Err(e)) => {
                        self.status = e;
                        self.extracting = false;
                        self.extract_rx = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        ctx.request_repaint(); // 완료 시점에 다시 그리도록
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.status = "Extraction thread stopped unexpectedly.".into();
                        self.extracting = false;
                        self.extract_rx = None;
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`
Expected: builds. (The `Extract` action is constructed in Task 6; until then `ListAction::Extract` is only matched, which is fine — no unreachable error. `extract_to_mp3` is now used.)

- [ ] **Step 6: Run tests**

Run: `cargo test`
Expected: PASS (no test regressions).

- [ ] **Step 7: Commit**

```bash
git add src/app.rs
git commit -m "feat: wire mp3 extraction trigger and channel polling"
```

---

## Task 6: MP3 button and ffmpeg banner in the UI

**Files:**
- Modify: `src/app.rs` — ffmpeg banner near the mpv banner; MP3 button in the saved-list row.

- [ ] **Step 1: Add the ffmpeg banner**

The mpv banner block in `update` is:

```rust
            if !self.mpv_available {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 60, 60),
                    "mpv is required: run `brew install mpv` in a terminal, then restart the app.",
                );
            }
```

Immediately after that block (before the `ui.separator();` that follows it), add:

```rust
            if !self.ffmpeg_available {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 60, 60),
                    "ffmpeg is required for MP3 extraction: run `brew install ffmpeg`, then restart the app.",
                );
            }
```

- [ ] **Step 2: Add the MP3 button to each saved-list row**

In the saved-list loop, the row currently starts:

```rust
                ui.horizontal(|ui| {
                    let play_btn = ui.add_enabled(self.mpv_available, egui::Button::new("Play"));
                    if play_btn.clicked() {
                        action = Some(ListAction::Play(i));
                    }
                    if ui.button("Edit").clicked() {
```

Insert the MP3 button between the Play handler and the Edit button, so it reads:

```rust
                ui.horizontal(|ui| {
                    let play_btn = ui.add_enabled(self.mpv_available, egui::Button::new("Play"));
                    if play_btn.clicked() {
                        action = Some(ListAction::Play(i));
                    }
                    let can_extract =
                        self.mpv_available && self.ffmpeg_available && !self.extracting;
                    if ui
                        .add_enabled(can_extract, egui::Button::new("MP3"))
                        .clicked()
                    {
                        action = Some(ListAction::Extract(i));
                    }
                    if ui.button("Edit").clicked() {
```

- [ ] **Step 3: Verify it compiles and tests pass**

Run: `cargo test`
Expected: PASS, no warnings about unused `ffmpeg_available`/`extracting`/`Extract`.

- [ ] **Step 4: Manual smoke test**

Run: `cargo run`
Expected: App launches. If ffmpeg is installed, each saved YouTube entry shows a clickable `MP3` button next to `Play`. Clicking it opens a folder picker; after choosing, the status shows `Extracting…`, then `Saved to: <folder>` once yt-dlp finishes (UI stays responsive throughout). If ffmpeg is missing, the red banner appears and MP3 buttons are disabled.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: add MP3 extraction button and ffmpeg banner to UI"
```

---

## Task 7: Update documentation

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Document the MP3 feature and ffmpeg requirement**

In `README.md`, add ffmpeg to the prerequisites/install instructions (alongside mpv) and add a short section describing the per-entry MP3 button: it extracts the YouTube audio to an mp3 in a folder you pick. Match the README's existing tone and structure. Example addition for the requirements area:

```markdown
- **ffmpeg** (for MP3 extraction): `brew install ffmpeg`
```

And a feature note:

```markdown
### Extract MP3

Each saved video has an **MP3** button. Click it, choose a destination folder, and
the audio is extracted to an `.mp3` file there (via yt-dlp + ffmpeg). The app stays
responsive while extracting; the status line shows progress and the final save folder.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: document MP3 extraction feature and ffmpeg requirement"
```

---

## Self-Review Notes

- **Spec coverage:** audio.rs module (Task 2-3), `build_extract_args` pure + tested (Task 2), `is_ffmpeg_available`/`yt_dlp_command` (Task 3), `extract_mp3` blocking + stderr in error (Task 3), per-entry MP3 button + enable conditions (Task 6), folder picker each time (Task 5), background thread + mpsc + repaint polling (Task 4-5), ffmpeg banner (Task 6), README/ffmpeg docs (Task 7). YAGNI items (local files, progress %, format choice, concurrent queue) intentionally excluded.
- **Types:** `extract_mp3 -> Result<String, String>` matches `Receiver<Result<String, String>>` and the `Ok(dir)`/`Err(e)` polling arms. `build_extract_args(&str, &Path)` consistent across definition, call site, and tests.
- **Visibility:** `EXTRA_PATHS`/`augmented_path` made `pub(crate)` in Task 1 before audio.rs imports them in Task 2.
