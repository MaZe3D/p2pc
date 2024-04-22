#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    // make `tokio::spawn` available
    let _enter = tokio_runtime.enter();

    // execute the runtime in seperate thread
    std::thread::spawn(move || {
        tokio_runtime.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        })
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    let mut app_name = "p2pc".to_string();

    if let Ok(instance_string) = std::env::var("INSTANCE") {
        app_name += "-";
        app_name += &instance_string;
    }

    eframe::run_native(
        &app_name,
        native_options,
        Box::new(|cc| Box::new(p2pc::App::new(cc))),
    ).unwrap();
}
