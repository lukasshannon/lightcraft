//! Phone layout around the same views and commands as the desktop host.
use crate::{
    LightcraftApp, panels,
    state::{RightPanel, ViewMode},
    theme::Tokens,
};
use serde_json::json;

fn button(ui: &mut egui::Ui, id: &str, label: &str, selected: bool) -> egui::Response {
    let response = ui.add(egui::Button::new(label).selected(selected).min_size(egui::vec2(42.0, 36.0)));
    crate::widgets::register(ui.ctx(), format!("mobile:{id}"), response.rect);
    response
}

pub fn show(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("mobile-header").exact_size(44.0).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        crate::screen_edges::row(ui, |ui| {
            ui.horizontal_centered(|ui| {
                crate::menubar::show_in_window(app, ui, 0.0);
                if button(ui, "import", "+ Photos", false).clicked() {
                    let _ = app.run("file.addPhotos", json!({}));
                }
                if button(ui, "export", "Export", false).clicked() {
                    if let Err(e) = app.run("dialog.export", json!({})) {
                        app.toast_error(ui.ctx(), e);
                    }
                }
                if ui.available_width() > 100.0 {
                    ui.label(egui::RichText::new("LightCraft").color(t.text_dim));
                }
            });
        });
    });
    egui::Panel::bottom("mobile-navigation").exact_size(46.0).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        crate::screen_edges::row(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if button(ui, "library", "Library", app.ui.left_panel).clicked() {
                    app.ui.left_panel = !app.ui.left_panel;
                    app.ui.presets = false;
                    app.ui.right = RightPanel::None;
                }
                if button(ui, "photos", "Photos", !app.ui.left_panel && app.ui.right == RightPanel::None && !app.ui.presets).clicked() {
                    app.ui.left_panel = false;
                    app.ui.presets = false;
                    app.ui.right = RightPanel::None;
                    let _ = app.run("view.photoGrid", json!({}));
                }
                if button(ui, "edit", "Edit", app.ui.right == RightPanel::Edit).clicked() {
                    app.ui.left_panel = false;
                    app.ui.presets = false;
                    let _ = app.run("view.detail", json!({}));
                    let _ = app.run("panel.edit", json!({}));
                }
                let tools = button(ui, "tools", "Tools", false);
                egui::Popup::menu(&tools).show(|ui| {
                    for (label, command) in [
                        ("Crop & Rotate", "panel.crop"),
                        ("Remove", "panel.remove"),
                        ("Masking", "panel.masking"),
                        ("Info", "panel.info"),
                        ("Keywords", "panel.keywords"),
                        ("Versions", "panel.versions"),
                        ("History", "panel.activity"),
                    ] {
                        if ui.button(label).clicked() {
                            app.ui.left_panel = false;
                            app.ui.presets = false;
                            let _ = app.run("view.detail", json!({}));
                            let _ = app.run(command, json!({}));
                            ui.close();
                        }
                    }
                });
                if button(ui, "presets", "Presets", app.ui.presets).clicked() {
                    app.ui.left_panel = false;
                    app.ui.right = RightPanel::None;
                    app.ui.presets = !app.ui.presets;
                }
            });
        });
    });
    if app.ui.left_panel {
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
            crate::widgets::register(ui.ctx(), "panel:left_panel", ui.max_rect());
            panels::left::show(app, ui);
        });
        return;
    }
    let tools = app.ui.right != RightPanel::None || app.ui.presets;
    if tools {
        let available = ui.available_size();
        let portrait = available.x < available.y;
        let panel = if portrait {
            egui::Panel::bottom("mobile-tools").exact_size((available.y * 0.45).clamp(120.0, 360.0))
        } else {
            egui::Panel::right("mobile-tools").exact_size(available.x.min(310.0).min(available.x * 0.46))
        };
        panel.frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
            if app.ui.presets {
                panels::presets::show(app, ui);
            } else {
                crate::widgets::register(ui.ctx(), "panel:right_panel", ui.max_rect());
                panels::right::show(app, ui);
            }
        });
    }
    panels::bottombar::show(app, ui);
    panels::compare::prepare(app);
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| match app.ui.view {
        ViewMode::PhotoGrid | ViewMode::SquareGrid => panels::grid::show(app, ui),
        ViewMode::Detail => panels::detail::show(app, ui),
        ViewMode::Compare => panels::compare::show_compare(app, ui),
        ViewMode::Survey => panels::compare::show_survey(app, ui),
        ViewMode::Reference => panels::compare::show_reference(app, ui),
        ViewMode::People => panels::people::show(app, ui),
    });
}
