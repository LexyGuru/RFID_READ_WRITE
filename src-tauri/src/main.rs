// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod nfc;
mod commands;

use commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            write_url_to_ntag216,
            write_text_to_ntag216,
            write_wifi_to_ntag216,
            write_bluetooth_to_ntag216,
            write_vcard_to_ntag216,
            write_email_to_ntag216,
            write_sms_to_ntag216,
            write_phone_to_ntag216,
            read_ntag216,
            read_ntag216_url,
            list_nfc_readers,
            check_nfc_available,
            set_ntag216_password,
            clear_ntag216_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

