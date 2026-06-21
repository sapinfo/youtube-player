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
