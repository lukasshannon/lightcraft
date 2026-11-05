//! Native Android host for LightCraft's Rust editor and engine.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod bridge;
#[cfg(target_os = "android")]
mod control;
#[cfg(target_os = "android")]
mod safe_area;
#[cfg(target_os = "android")]
mod services;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let Some(files) = app.internal_data_path() else {
        log::error!("No Android app data directory");
        return;
    };
    let bridge = match bridge::Bridge::new(&files) {
        Ok(bridge) => bridge,
        Err(error) => {
            log::error!("Cannot prepare Android storage: {error}");
            return;
        }
    };
    let mut options = eframe::NativeOptions {
        android_app: Some(app),
        persistence_path: Some(files.join("ui.ron")),
        viewport: egui::ViewportBuilder::default().with_app_id("app.lightcraft.android").with_title("LightCraft"),
        ..Default::default()
    };
    // Some Android GL drivers expose 16 KiB uniform buffers. Eframe's desktop
    // defaults ask for 64 KiB, which would prevent the window from opening.
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup {
        create.device_descriptor = std::sync::Arc::new(|adapter| eframe::wgpu::DeviceDescriptor {
            label: Some("LightCraft Android renderer"),
            required_limits: adapter.limits(),
            ..Default::default()
        });
    }
    let started = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "LightCraft",
            options,
            Box::new(move |cc| {
                let mut session = lightcraft_engine::Session::new().with_fs().with_system_clock();
                session.open_library(files.join("library"), false).map_err(|e| std::io::Error::other(e.to_string()))?;
                let mut editor = lightcraft_ui_egui::LightcraftApp::new(session, services::services(bridge, &cc.egui_ctx));
                // Leave room for the first editing controls on a phone. The saved setting
                // below still lets the user keep a histogram if they enable it.
                editor.ui.histogram = false;
                if let Ok(bytes) = std::fs::read(files.join("preferences.json"))
                    && let Ok(prefs) = serde_json::from_slice(&bytes)
                {
                    editor.ui = prefs;
                }
                editor.integrated_titlebar = false;
                editor.mobile_host = true;
                lightcraft_ui_egui::panels::empty_library::open_on_empty_library(&mut editor);
                editor.ui.left_panel = false;
                editor.ui.right = lightcraft_ui_egui::state::RightPanel::None;
                editor.ui.filmstrip = false;
                if let Some(rx) = control::start(&files, &cc.egui_ctx) {
                    editor = editor.with_control(rx);
                }
                let safe_area = safe_area::SafeArea::new(files.join("bridge/safe-area.json"));
                Ok(Box::new(AndroidEditor { editor, safe_area, files, last_prefs: Vec::new(), last_save: std::time::Instant::now() }))
            }),
        )
    }));
    if !matches!(started, Ok(Ok(()))) {
        log::error!("LightCraft could not run: {started:?}");
    }
}

#[cfg(target_os = "android")]
struct AndroidEditor {
    editor: lightcraft_ui_egui::LightcraftApp,
    safe_area: safe_area::SafeArea,
    files: std::path::PathBuf,
    last_prefs: Vec<u8>,
    last_save: std::time::Instant,
}

#[cfg(target_os = "android")]
impl AndroidEditor {
    fn save_preferences(&mut self) {
        if let Ok(bytes) = serde_json::to_vec(&self.editor.ui)
            && bytes != self.last_prefs
        {
            match bridge::atomic_write(&self.files.join("preferences.json"), &bytes) {
                Ok(()) => self.last_prefs = bytes,
                Err(e) => log::error!("Cannot save preferences: {e}"),
            }
        }
        self.editor.session.persist_if_dirty();
        self.editor.session.save_view();
    }
}

#[cfg(target_os = "android")]
impl eframe::App for AndroidEditor {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.safe_area.apply(ctx, raw);
        self.editor.raw_input_hook(raw);
    }
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.editor.logic(ctx);
        let notice = self.files.join("bridge/notice.json");
        if let Ok(message) = std::fs::read_to_string(&notice) {
            let _ = std::fs::remove_file(notice);
            self.editor.toast_error(ctx, message);
        }
        if self.last_save.elapsed() >= std::time::Duration::from_secs(2) {
            self.save_preferences();
            self.last_save = std::time::Instant::now();
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.safe_area.prepare_ui(ui.ctx());
        let rect = ui.ctx().content_rect();
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.set_clip_rect(rect);
        self.editor.ui(&mut child);
    }
    fn on_exit(&mut self) {
        self.save_preferences();
        if let Err(e) = self.editor.session.close_library() {
            log::error!("Closing library: {e}");
        }
        self.editor.shutdown(std::time::Duration::from_secs(2));
    }
}
