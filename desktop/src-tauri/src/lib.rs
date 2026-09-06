use std::path::PathBuf;

use local::{ZygoLocalConfig, ZygoLocalService, DEFAULT_DATABASE_BUSY_TIMEOUT};
use tauri_specta::{collect_commands, Builder};
use zygo_core::ZygoConfig;

mod commands;
mod error;

use commands::{load_data, sync, watch_logs};

const TYPESCRIPT_BINDINGS_PATH: &str = "../src/bindings.ts";

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[specta::specta]
fn greet(name: &str, title: &str) -> String {
    format!("Hello, {} {}! You've been greeted from Rust!", name, title)
}

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![greet, load_data, sync, watch_logs])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let service = tauri::async_runtime::block_on(ZygoLocalService::new(ZygoLocalConfig {
        base: ZygoConfig::new(1),
        database_busy_timeout: DEFAULT_DATABASE_BUSY_TIMEOUT,
    }))
    .expect("failed to start the local Zygo service");
    let specta = specta_builder();

    tauri::Builder::default()
        .manage(service)
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
