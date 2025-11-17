#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod nfc;

use nfc::{NfcReader, Ntag216};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Response {
  message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PasswordConfig {
  password: Vec<u8>,
  pack: Vec<u8>,
  #[serde(alias = "authLimit", alias = "auth_limit")]
  auth_limit: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct TextConfig {
  text: String,
  language: String,
}

/// URL írása NTAG216 címkére
#[tauri::command]
fn write_url(url: String, password: Option<Vec<u8>>) -> Result<String, String> {
  println!("📝 write_url CALLED");
  println!("  📥 URL: {}", url);
  if let Some(ref pwd) = password {
    println!("  📥 Password: {:?} (len: {})", pwd, pwd.len());
  }
  
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let pwd_array = password.as_ref().map(|p| {
        if p.len() != 4 {
          return Err(format!("A password pontosan 4 byte kell legyen! (kapott: {})", p.len()));
        }
        Ok([p[0], p[1], p[2], p[3]])
      }).transpose()?;
      
      ntag.write_ndef_url_with_password(&card, &url, pwd_array.as_ref())
        .map_err(|e| format!("Írási hiba: {}", e))?;
      println!("  ✅ URL sikeresen írva");
      Ok(format!("Sikeresen írtam az URL-t az NTAG216 címkére: {}", url))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// URL olvasása NTAG216 címkéből
#[tauri::command]
fn read_url() -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      match ntag.read_ndef(&card)
        .map_err(|e| format!("Olvasási hiba: {}", e))?
      {
        Some(url) => Ok(url),
        None => Err("Nem található NDEF URL üzenet a címkén".to_string()),
      }
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// NDEF Text Record írása
#[tauri::command]
fn write_text(text: String, language: String, password: Option<Vec<u8>>) -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let pwd_array = password.as_ref().map(|p| {
        if p.len() != 4 {
          return Err(format!("A password pontosan 4 byte kell legyen! (kapott: {})", p.len()));
        }
        Ok([p[0], p[1], p[2], p[3]])
      }).transpose()?;
      
      ntag.write_ndef_text_with_password(&card, &text, &language, pwd_array.as_ref())
        .map_err(|e| format!("Írási hiba: {}", e))?;
      Ok(format!("Sikeresen írtam a szöveget az NTAG216 címkére"))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// NDEF Text Record olvasása
#[tauri::command]
fn read_text() -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      match ntag.read_ndef_text(&card)
        .map_err(|e| format!("Olvasási hiba: {}", e))?
      {
        Some((text, lang)) => Ok(format!("[{}] {}", lang, text)),
        None => Err("Nem található NDEF Text üzenet a címkén".to_string()),
      }
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// NDEF vCard írása
#[tauri::command]
fn write_vcard(vcard: String, password: Option<Vec<u8>>) -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let pwd_array = password.as_ref().map(|p| {
        if p.len() != 4 {
          return Err(format!("A password pontosan 4 byte kell legyen! (kapott: {})", p.len()));
        }
        Ok([p[0], p[1], p[2], p[3]])
      }).transpose()?;
      
      ntag.write_ndef_vcard_with_password(&card, &vcard, pwd_array.as_ref())
        .map_err(|e| format!("Írási hiba: {}", e))?;
      Ok(format!("Sikeresen írtam a vCard-ot az NTAG216 címkére"))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// NDEF vCard olvasása
#[tauri::command]
fn read_vcard() -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      match ntag.read_ndef_vcard(&card)
        .map_err(|e| format!("Olvasási hiba: {}", e))?
      {
        Some(vcard) => Ok(vcard),
        None => Err("Nem található NDEF vCard üzenet a címkén".to_string()),
      }
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// Password beállítása
#[tauri::command]
fn set_password(password: Vec<u8>, pack: Vec<u8>, auth_limit: u8) -> Result<String, String> {
  println!("🔐 set_password CALLED");
  println!("  📥 password: {:?} (len: {})", password, password.len());
  println!("  📥 pack: {:?} (len: {})", pack, pack.len());
  println!("  📥 auth_limit: {}", auth_limit);
  
  if password.len() != 4 {
    println!("  ❌ Password hossz hiba: {} != 4", password.len());
    return Err(format!("A password pontosan 4 byte kell legyen! (kapott: {})", password.len()));
  }
  if pack.len() != 2 {
    println!("  ❌ PACK hossz hiba: {} != 2", pack.len());
    return Err(format!("A PACK pontosan 2 byte kell legyen! (kapott: {})", pack.len()));
  }
  
  println!("  ✅ Paraméterek validálva");
  
  println!("  🔌 NFC olvasó inicializálása...");
  let reader = NfcReader::new()
    .map_err(|e| {
      println!("  ❌ NFC olvasó hiba: {}", e);
      format!("NFC olvasó inicializálási hiba: {}", e)
    })?;
  println!("  ✅ NFC olvasó inicializálva");
  
  println!("  📡 Címke csatlakoztatása...");
  let card = reader.connect()
    .map_err(|e| {
      println!("  ❌ Csatlakozási hiba: {}", e);
      format!("Csatlakozási hiba: {}", e)
    })?;
  println!("  ✅ Címke csatlakoztatva");
  
  let ntag = Ntag216;
  
  println!("  🔍 NTAG216 típus ellenőrzése...");
  // Próbáljuk meg password nélkül, ha nem működik, akkor lehet hogy már password védett
  match ntag.check_type(&card) {
    Ok(true) => {
      println!("  ✅ NTAG216 címke megerősítve");
      let pwd_array: [u8; 4] = [password[0], password[1], password[2], password[3]];
      let pack_array: [u8; 2] = [pack[0], pack[1]];
      
      println!("  📝 Password beállítása...");
      println!("    Password: {:02X?}", pwd_array);
      println!("    PACK: {:02X?}", pack_array);
      println!("    Auth Limit: {}", auth_limit);
      
      ntag.set_password(&card, &pwd_array, &pack_array, auth_limit)
        .map_err(|e| {
          println!("  ❌ Password beállítási hiba: {}", e);
          format!("Password beállítási hiba: {}", e)
        })?;
      println!("  ✅ Password sikeresen beállítva");
      Ok(format!("Sikeresen beállítottam a password védelmet"))
    }
    Ok(false) => {
      println!("  ❌ Ez nem egy NTAG216 címke!");
      Err("Ez nem egy NTAG216 címke!".to_string())
    },
    Err(e) => {
      // Ha password védelem aktív, akkor lehet hogy már be van állítva
      let error_msg = format!("{}", e);
      if error_msg.contains("SW1=0x63") || error_msg.contains("Password védelem aktív") {
        println!("  ⚠️ Password védelem aktív lehet, próbáljuk meg password-dal...");
        let pwd_array: [u8; 4] = [password[0], password[1], password[2], password[3]];
        match ntag.check_type_with_password(&card, Some(&pwd_array)) {
          Ok(true) => {
            println!("  ✅ NTAG216 címke megerősítve (password-dal)");
            println!("  💡 A password már be van állítva, vagy módosítsd a 'Password Eltávolítása' gombbal.");
            Err("A password már be van állítva a címkére! Használd a 'Password Eltávolítása' gombot, ha újra be szeretnéd állítani.".to_string())
          }
          Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
          Err(e2) => {
            println!("  ❌ Címke ellenőrzési hiba (password-dal is): {}", e2);
            Err(format!("Címke ellenőrzési hiba: {}", e2))
          }
        }
      } else {
        println!("  ❌ Címke ellenőrzési hiba: {}", e);
        Err(format!("Címke ellenőrzési hiba: {}", e))
      }
    },
  }
}

/// Password védelem eltávolítása
#[tauri::command]
fn remove_password() -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      ntag.remove_password(&card)
        .map_err(|e| format!("Password eltávolítási hiba: {}", e))?;
      Ok(format!("Sikeresen eltávolítottam a password védelmet"))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// Read-only mód beállítása (VISSZAFORDÍTHATATLAN!)
#[tauri::command]
fn set_read_only() -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      ntag.set_read_only(&card)
        .map_err(|e| format!("Read-only beállítási hiba: {}", e))?;
      Ok(format!("⚠️ Read-only mód beállítva! VISSZAFORDÍTHATATLAN!"))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// Konfiguráció olvasása
#[tauri::command]
fn read_config() -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let config = ntag.read_config(&card)
        .map_err(|e| format!("Konfiguráció olvasási hiba: {}", e))?;
      
      let pwd_str = format!("{:02X}{:02X}{:02X}{:02X}", 
        config.password[0], config.password[1], config.password[2], config.password[3]);
      let pack_str = format!("{:02X}{:02X}", config.pack[0], config.pack[1]);
      
      Ok(format!(
        "Password: {}\nPACK: {}\nAuth Limit: {}\nPassword Védett: {}\nRead-Only: {}\nLocked: {}",
        pwd_str,
        pack_str,
        config.auth_limit,
        if config.password_protected { "Igen" } else { "Nem" },
        if config.read_only { "Igen" } else { "Nem" },
        if config.locked { "Igen ⚠️" } else { "Nem" }
      ))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// NDEF üzenet törlése
#[tauri::command]
fn clear_ndef(password: Option<Vec<u8>>) -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let pwd_array = password.as_ref().map(|p| {
        if p.len() != 4 {
          return Err(format!("A password pontosan 4 byte kell legyen! (kapott: {})", p.len()));
        }
        Ok([p[0], p[1], p[2], p[3]])
      }).transpose()?;
      
      ntag.clear_ndef_with_password(&card, pwd_array.as_ref())
        .map_err(|e| format!("Törlési hiba: {}", e))?;
      Ok(format!("Sikeresen töröltem az NDEF üzenetet"))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// Raw byte írása
#[tauri::command]
fn write_raw(start_block: u8, data: Vec<u8>, password: Option<Vec<u8>>) -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let pwd_array = password.as_ref().map(|p| {
        if p.len() != 4 {
          return Err(format!("A password pontosan 4 byte kell legyen! (kapott: {})", p.len()));
        }
        Ok([p[0], p[1], p[2], p[3]])
      }).transpose()?;
      
      ntag.write_raw_bytes_with_password(&card, start_block, &data, pwd_array.as_ref())
        .map_err(|e| format!("Írási hiba: {}", e))?;
      Ok(format!("Sikeresen írtam {} byte-ot block {}-tól", data.len(), start_block))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// Raw byte olvasása
#[tauri::command]
fn read_raw(start_block: u8, count: u8) -> Result<String, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  let card = reader.connect()
    .map_err(|e| format!("Csatlakozási hiba: {}", e))?;
  
  let ntag = Ntag216;
  
  match ntag.check_type(&card) {
    Ok(true) => {
      let data = ntag.read_raw_bytes(&card, start_block, count)
        .map_err(|e| format!("Olvasási hiba: {}", e))?;
      
      let hex_str: Vec<String> = data.iter().map(|b| format!("{:02X}", b)).collect();
      Ok(hex_str.join(" "))
    }
    Ok(false) => Err("Ez nem egy NTAG216 címke!".to_string()),
    Err(e) => Err(format!("Címke ellenőrzési hiba: {}", e)),
  }
}

/// NFC olvasók listázása
#[tauri::command]
fn list_readers() -> Result<Vec<String>, String> {
  let reader = NfcReader::new()
    .map_err(|e| format!("NFC olvasó inicializálási hiba: {}", e))?;
  
  reader.list_readers()
    .map_err(|e| format!("Olvasók listázási hiba: {}", e))
}

fn main() {
  println!("🚀 NTAG216 NFC Alkalmazás indítása...");
  
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      write_url,
      read_url,
      write_text,
      read_text,
      write_vcard,
      read_vcard,
      set_password,
      remove_password,
      set_read_only,
      read_config,
      clear_ndef,
      write_raw,
      read_raw,
      list_readers
    ])
    .setup(|_app| {
      println!("✅ Tauri alkalmazás inicializálva");
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
