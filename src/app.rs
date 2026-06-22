use std::sync::mpsc::Receiver;

use eframe::egui;

use crate::audio;
use crate::player::{self, WindowSize};
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
    ffmpeg_available: bool,
    status: String,
    extracting: bool,
    extract_rx: Option<Receiver<Result<String, String>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: store::load(),
            form: Form::default(),
            mpv_available: player::is_mpv_available(),
            ffmpeg_available: audio::is_ffmpeg_available(),
            status: String::new(),
            extracting: false,
            extract_rx: None,
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
                        // 추출 완료 시점을 반영하도록 주기적으로 다시 그린다.
                        // 매 프레임 spin 하지 않게 200ms 간격으로 폴링.
                        ctx.request_repaint_after(std::time::Duration::from_millis(200));
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
            ui.heading("YouTube Player");

            if !self.mpv_available {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 60, 60),
                    "mpv is required: run `brew install mpv` in a terminal, then restart the app.",
                );
            }

            if !self.ffmpeg_available {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 60, 60),
                    "ffmpeg is required for MP3 extraction: run `brew install ffmpeg`, then restart the app.",
                );
            }

            ui.separator();

            // --- playback settings ---
            ui.label("Playback settings");
            let mut changed = false;
            changed |= ui.checkbox(&mut self.data.settings.ontop, "Always on top").changed();
            egui::ComboBox::from_label("Window size")
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

            // --- local file ---
            ui.label("Local video file");
            if ui
                .add_enabled(self.mpv_available, egui::Button::new("Play local file…"))
                .clicked()
            {
                self.play_local_file();
            }

            ui.separator();

            // --- saved list ---
            ui.label("Saved videos");
            let mut action: Option<ListAction> = None;
            let count = self.data.entries.len();
            for (i, entry) in self.data.entries.iter().enumerate() {
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
                        action = Some(ListAction::Edit(i));
                    }
                    if ui.button("Delete").clicked() {
                        action = Some(ListAction::Delete(i));
                    }
                    if ui.add_enabled(i > 0, egui::Button::new("Up")).clicked() {
                        action = Some(ListAction::Up(i));
                    }
                    if ui.add_enabled(i + 1 < count, egui::Button::new("Down")).clicked() {
                        action = Some(ListAction::Down(i));
                    }
                    ui.label(if entry.name.is_empty() { &entry.url } else { &entry.name });
                });
            }

            if let Some(a) = action {
                self.handle_list_action(a);
            }

            ui.separator();

            // --- add/edit form ---
            ui.label(if self.form.editing.is_some() { "Edit" } else { "Add" });
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.form.name);
            });
            ui.horizontal(|ui| {
                ui.label("URL");
                ui.text_edit_singleline(&mut self.form.url);
            });
            ui.horizontal(|ui| {
                let label = if self.form.editing.is_some() { "Save" } else { "Add" };
                if ui.button(label).clicked() {
                    self.submit_form();
                }
                if self.form.editing.is_some() && ui.button("Cancel").clicked() {
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
    Extract(usize),
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
                        self.status = "Invalid YouTube URL.".into();
                    } else {
                        let url = e.url.clone();
                        let settings = self.data.settings.clone();
                        match player::play(&url, &settings) {
                            Ok(()) => self.status = "Playing…".into(),
                            Err(e) => self.status = e,
                        }
                    }
                }
            }
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

    /// 네이티브 파일 선택기로 동영상 파일을 골라 mpv로 바로 재생한다.
    fn play_local_file(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Select a video file")
            .add_filter(
                "Video",
                &[
                    "mp4", "mov", "mkv", "avi", "webm", "m4v", "flv", "wmv", "mpg", "mpeg", "ts",
                    "m2ts",
                ],
            )
            .pick_file();
        if let Some(path) = picked {
            let path_str = path.to_string_lossy().to_string();
            let settings = self.data.settings.clone();
            match player::play(&path_str, &settings) {
                Ok(()) => self.status = "Playing…".into(),
                Err(e) => self.status = e,
            }
        }
    }

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

    fn submit_form(&mut self) {
        let name = self.form.name.trim().to_string();
        let url_value = self.form.url.trim().to_string();
        if url_value.is_empty() || !url::validate(&url_value) {
            self.status = "Invalid YouTube URL.".into();
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
