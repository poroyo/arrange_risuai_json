mod panels;

#[cfg(not(target_arch = "wasm32"))]

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000., 1000.)),
        ..Default::default()
    };

    eframe::run_native(
        "Test App",
        options,
        Box::new(|cc| Box::new(panels::BigFrame::_new(cc)))
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();
    // tracing_wasm::set_as_global_default();
    // console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new().start(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(panels::BigFrame::_new(cc)))
        )
        .await
        .expect("failed to start eframe");
    });
}
