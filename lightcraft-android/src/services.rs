use crate::bridge::Bridge;
use lightcraft_ui_egui::Services;
use std::sync::Arc;

pub fn services(bridge: Bridge, ctx: &egui::Context) -> Services {
    let writer = bridge.clone();
    let shared = bridge.clone();
    let repaint = ctx.clone();
    Services {
        picker: Some(Box::new(move |request| {
            let (tx, rx) = std::sync::mpsc::channel();
            let bridge = bridge.clone();
            let ctx = repaint.clone();
            std::thread::Builder::new()
                .name("Android document picker".into())
                .spawn(move || {
                    let paths = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| bridge.pick(request))) {
                        Ok(Ok(paths)) => paths,
                        Ok(Err(error)) => {
                            log::error!("Android picker: {error}");
                            let _ = crate::bridge::atomic_write(&bridge.files.join("bridge/notice.json"), error.as_bytes());
                            Vec::new()
                        }
                        Err(_) => {
                            log::error!("Android picker failed unexpectedly");
                            Vec::new()
                        }
                    };
                    let _ = tx.send(paths);
                    ctx.request_repaint();
                })
                .map_err(|e| e.to_string())?;
            Ok(rx)
        })),
        write: Some(Box::new(move |path, bytes| writer.write(path, bytes))),
        write_shared: Some(Arc::new(move |path, bytes| shared.write(path, bytes))),
        png: Some(Box::new(|img| {
            lightcraft_codecs::encode_png(&lightcraft_codecs::EncodeImage::rgba8(img), &lightcraft_codecs::EncodeMeta::default()).unwrap_or_default()
        })),
        ..Default::default()
    }
}
