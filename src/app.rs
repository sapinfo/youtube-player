use eframe::egui;

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
