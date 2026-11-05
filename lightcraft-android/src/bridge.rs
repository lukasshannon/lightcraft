//! SAF requests are exchanged in app-private storage; provider I/O runs off the UI thread.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use lightcraft_ui_egui::pick::{PickKind, PickRequest};
use serde_json::{Value, json};

static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct Bridge {
    pub files: PathBuf,
    root: PathBuf,
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension(format!("tmp-{}", NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)));
    let result = (|| {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

impl Bridge {
    pub fn new(files: &Path) -> Result<Self, String> {
        let root = files.join("bridge");
        for dir in [root.join("requests"), root.join("replies"), files.join("imports"), files.join("exports")] {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        Ok(Self { files: files.to_path_buf(), root })
    }

    pub fn call(&self, request: Value, timeout: Duration) -> Result<Value, String> {
        let id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        let clock = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
        let name = format!("{clock}-{id}.json");
        let request_path = self.root.join("requests").join(&name);
        let response_path = self.root.join("replies").join(&name);
        atomic_write(&request_path, &serde_json::to_vec(&request).map_err(|e| e.to_string())?)?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            match std::fs::read(&response_path) {
                Ok(bytes) => {
                    let _ = std::fs::remove_file(&response_path);
                    let response: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    if response["kind"] == "error" {
                        return Err(response["message"].as_str().unwrap_or("Android storage failed").into());
                    }
                    return Ok(response);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => std::thread::sleep(Duration::from_millis(80)),
                Err(e) => return Err(e.to_string()),
            }
        }
        let _ = std::fs::remove_file(request_path);
        Err("Android storage timed out. Please try again.".into())
    }

    pub fn pick(&self, req: PickRequest) -> Result<Vec<String>, String> {
        let request = match req.kind {
            PickKind::Files | PickKind::File => {
                json!({"kind":"open", "multiple": req.kind == PickKind::Files})
            }
            PickKind::Save => {
                json!({"kind":"save", "suggested":req.file_name.unwrap_or_else(|| "Photo.jpg".into())})
            }
            PickKind::Folder => {
                let export = req.title.as_deref().is_some_and(|t| t == "Android export folder");
                json!({"kind":"folder", "export":export})
            }
        };
        parse_paths(&self.call(request, Duration::from_secs(1800))?)
    }

    pub fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        let target = Path::new(path);
        if !target.starts_with(&self.files) || target.components().any(|c| c == std::path::Component::ParentDir) {
            return Err("Choose an Android storage destination with Choose… first.".into());
        }
        atomic_write(target, bytes)?;
        if target.starts_with(self.files.join("exports")) {
            self.call(json!({"kind":"write", "path":path}), Duration::from_secs(180))?;
        }
        Ok(())
    }
}

fn parse_paths(value: &Value) -> Result<Vec<String>, String> {
    match value["kind"].as_str() {
        Some("cancelled") => Ok(Vec::new()),
        Some("save") => value["path"].as_str().map(|p| vec![p.to_owned()]).ok_or_else(|| "No save path returned".into()),
        Some("paths") => value["paths"]
            .as_array()
            .ok_or("No file paths returned")?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or_else(|| "Invalid file path returned".into()))
            .collect(),
        _ => Err("Unknown Android picker response".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picker_cancellation_and_malformed_results() {
        assert!(parse_paths(&json!({"kind":"cancelled"})).is_ok_and(|p| p.is_empty()));
        assert!(parse_paths(&json!({"kind":"paths", "paths":[42]})).is_err());
        assert!(parse_paths(&json!({"kind":"save"})).is_err());
    }
    #[test]
    fn importing_returns_persistent_paths_without_deleting_originals() {
        let p = "/data/user/0/app.lightcraft.android/files/imports/one/image.dng";
        assert_eq!(parse_paths(&json!({"kind":"paths", "paths":[p]})), Ok(vec![p.to_string()]));
    }
}
