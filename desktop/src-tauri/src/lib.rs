use std::path::PathBuf;
use tauri_specta::{collect_commands, Builder};

const TYPESCRIPT_BINDINGS_PATH: &str = "../src/bindings.ts";

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[specta::specta]
fn greet(name: &str, title: &str) -> String {
    format!("Hello, {} {}! You've been greeted from Rust!", name, title)
}

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![greet])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta = specta_builder();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Exports TypeScript bindings for the Tauri commands.
/// Used in the gen binary so we can regen as needed.
pub fn export_typescript_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TYPESCRIPT_BINDINGS_PATH);
    specta_builder().export(specta_typescript::Typescript::default(), output)?;
    Ok(())
}
