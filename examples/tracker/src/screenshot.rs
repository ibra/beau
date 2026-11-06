//! `--screenshot`: render for a moment, save a PNG, exit. Used to review the look.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::{self, ColorImage};

/// Seconds to let the UI settle (fonts, entry animations) before capturing.
const SETTLE_SECONDS: f64 = 1.2;

pub struct ScreenshotJob {
    path: PathBuf,
    start: Option<f64>,
    requested: bool,
    done: Arc<AtomicBool>,
}

impl ScreenshotJob {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            start: None,
            requested: false,
            done: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Drive the job; call once per frame.
    pub fn tick(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();
        if self.done.load(Ordering::Acquire) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let now = ctx.input(|i| i.time);
        let start = *self.start.get_or_insert(now);
        if !self.requested && now - start >= SETTLE_SECONDS {
            self.requested = true;
            let path = self.path.clone();
            let done = self.done.clone();
            let ctx2 = ctx.clone();
            ctx.request_screenshot(move |image: Arc<ColorImage>| {
                if let Err(e) = save_png(&path, &image) {
                    eprintln!("tracker: failed to save screenshot: {e}");
                } else {
                    eprintln!("tracker: saved {}", path.display());
                }
                done.store(true, Ordering::Release);
                ctx2.request_repaint();
            });
        }
    }
}

fn save_png(path: &std::path::Path, image: &ColorImage) -> image::ImageResult<()> {
    let [w, h] = image.size;
    let bytes: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|c| c.to_srgba_unmultiplied())
        .collect();
    image::save_buffer(path, &bytes, w as u32, h as u32, image::ColorType::Rgba8)
}
