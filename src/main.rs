mod app;
mod audio;
mod player;
mod store;
mod url;

use eframe::egui;

/// Register a Korean-capable system font as a fallback so user-entered Korean
/// (e.g. entry names) renders instead of tofu boxes. The English UI keeps
/// egui's default font for Latin text; Korean glyphs fall back to this font.
fn install_korean_font(ctx: &egui::Context) {
    let candidates = [
        "/System/Library/Fonts/Supplemental/AppleGothic.ttf",
        "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("korean".to_owned(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("korean".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("korean".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "YouTube Player",
        options,
        Box::new(|cc| {
            install_korean_font(&cc.egui_ctx);
            Ok(Box::<app::App>::default())
        }),
    )
}
