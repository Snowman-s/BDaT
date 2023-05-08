// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::PathBuf};

use tauri::{api::dialog::blocking::FileDialogBuilder, CustomMenuItem, Menu, MenuItem, Submenu};

#[tauri::command]
fn find_binary_file_path() -> Option<PathBuf> {
    FileDialogBuilder::new().pick_file()
}

#[tauri::command]
fn load_binary_file(path: String) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| e.to_string())
}

fn main() {
    let open = CustomMenuItem::new("open".to_string(), "開く");
    let quit = CustomMenuItem::new("quit".to_string(), "アプリを終了する");
    let submenu = Submenu::new(
        "ファイル",
        Menu::new()
            .add_item(open)
            .add_native_item(MenuItem::Separator)
            .add_item(quit),
    );
    let menu = Menu::new().add_submenu(submenu);

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "open" => {
                let window = event.window();
                window.emit("open", ()).unwrap();
            }
            "quit" => std::process::exit(0),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            load_binary_file,
            find_binary_file_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
