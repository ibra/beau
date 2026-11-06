//! beau reference app: a small task tracker in the spirit of Linear.
//!
//! Native: `cargo run -p tracker [-- --dark] [--dialog] [--stock] [--screenshot out.png]`
//! Web:    `trunk serve` from this directory.

mod app;
mod data;
mod glyphs;
#[cfg(not(target_arch = "wasm32"))]
mod screenshot;

/// Startup options (parsed from the command line on native).
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Start in dark mode.
    pub dark: bool,
    /// Open the new-task dialog at startup.
    pub dialog: bool,
    /// Don't install beau: render with stock egui for before/after comparisons.
    pub stock: bool,
    /// Save a screenshot to this path after startup, then exit.
    #[cfg(not(target_arch = "wasm32"))]
    pub screenshot: Option<std::path::PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let mut opts = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dark" => opts.dark = true,
            "--dialog" => opts.dialog = true,
            "--stock" => opts.stock = true,
            "--screenshot" => opts.screenshot = args.next().map(Into::into),
            other => eprintln!("tracker: ignoring unknown argument {other:?}"),
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("beau · tracker")
            .with_inner_size([1180.0, 740.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "beau-tracker",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::TrackerApp::new(cc, opts)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let canvas = document
            .get_element_by_id("beau_canvas")
            .expect("missing #beau_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#beau_canvas is not a canvas");
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::TrackerApp::new(cc, Options::default())))),
            )
            .await;
        if let Err(e) = result {
            eframe::web_sys::console::error_1(&e);
        }
    });
}
