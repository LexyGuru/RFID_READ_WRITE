use crate::nfc::{NFCReader, NTAG216};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct NFCReaderInfo {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagInfo {
    uid: Vec<u8>,
    url: Option<String>,
    text: Option<String>,
    language: Option<String>,
    record_type: Option<String>, // "uri", "text", "wifi", "bluetooth", "vcard", "email", "sms", "phone", "unknown"
    // WiFi adatok
    wifi_ssid: Option<String>,
    wifi_security: Option<String>,
    // Bluetooth adatok
    bluetooth_mac: Option<String>,
    // vCard adatok
    vcard_name: Option<String>,
    vcard_phone: Option<String>,
    vcard_email: Option<String>,
    vcard_organization: Option<String>,
    // Email adatok
    email_address: Option<String>,
    email_subject: Option<String>,
    email_body: Option<String>,
    // SMS adatok
    sms_phone: Option<String>,
    sms_message: Option<String>,
    // Phone adatok
    phone_number: Option<String>,
}

#[tauri::command]
pub async fn list_nfc_readers() -> Result<Vec<String>, String> {
    let reader = NFCReader::new().map_err(|e| format!("{:?}", e))?;
    reader.list_readers().map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn check_nfc_available() -> Result<bool, String> {
    let reader = NFCReader::new().map_err(|e| format!("{:?}", e))?;
    let readers = reader.list_readers().map_err(|e| format!("{:?}", e))?;
    Ok(!readers.is_empty())
}

#[tauri::command]
pub async fn write_url_to_ntag216(url: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    
    // Várunk egy címkére (5 másodperc timeout)
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}. Győződj meg róla, hogy a címke az olvasón van!", e))?;

    let ntag = NTAG216::new(card);
    
    // UID olvasása
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    // Ellenőrizzük a CC blokkot (debug)
    match ntag.read_block(0x03) {
        Ok(cc) => {
            eprintln!("CC block (0x03) current value: {:02X?}", cc);
        }
        Err(e) => {
            eprintln!("Could not read CC block: {:?}", e);
        }
    }
    
    // URL írása
    ntag.write_ndef_uri(&url).map_err(|e| {
        format!("Write error: {:?}\n\nLehetséges okok:\n- A címke read-only módban van\n- A címke nem NTAG216 típusú\n- Az olvasó nem támogatja az írást\n- A címke eltávolodott az olvasóról\n\nTipp: Próbáld meg először olvasni a címkét, hogy lássuk mi van rajta!", e)
    })?;
    
    // Ellenőrizzük, hogy tényleg megíródott-e
    match ntag.read_ndef_uri() {
        Ok(read_url) => {
            if read_url != url {
                return Err(format!(
                    "Az írás nem sikerült teljesen! Írt: '{}', Olvasott: '{}'",
                    url, read_url
                ));
            }
        }
        Err(e) => {
            return Err(format!(
                "Az írás után nem sikerült visszaolvasni az adatot: {:?}",
                e
            ));
        }
    }
    
    Ok(TagInfo {
        uid,
        url: Some(url),
        text: None,
        language: None,
        record_type: Some("uri".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn write_text_to_ntag216(text: String, language: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    
    // Várunk egy címkére (5 másodperc timeout)
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}. Győződj meg róla, hogy a címke az olvasón van!", e))?;

    let ntag = NTAG216::new(card);
    
    // UID olvasása
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    // Text írása
    ntag.write_ndef_text(&text, &language).map_err(|e| {
        format!("Write error: {:?}\n\nLehetséges okok:\n- A címke read-only módban van\n- A címke nem NTAG216 típusú\n- Az olvasó nem támogatja az írást\n- A címke eltávolodott az olvasóról", e)
    })?;
    
    // Ellenőrizzük, hogy tényleg megíródott-e
    match ntag.read_ndef_text() {
        Ok((read_text, read_lang)) => {
            if read_text != text {
                return Err(format!(
                    "Az írás nem sikerült teljesen! Írt: '{}', Olvasott: '{}'",
                    text, read_text
                ));
            }
        }
        Err(e) => {
            return Err(format!(
                "Az írás után nem sikerült visszaolvasni az adatot: {:?}",
                e
            ));
        }
    }
    
    Ok(TagInfo {
        uid,
        url: None,
        text: Some(text),
        language: Some(language),
        record_type: Some("text".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn read_ntag216() -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    
    // Várunk egy címkére (5 másodperc timeout)
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    
    // UID olvasása
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    // Próbáljuk meg URI-ként olvasni
    if let Ok(url) = ntag.read_ndef_uri() {
        // Ellenőrizzük, hogy email, SMS vagy phone URI-e
        let record_type = if url.starts_with("mailto:") {
            "email"
        } else if url.starts_with("sms:") {
            "sms"
        } else if url.starts_with("tel:") {
            "phone"
        } else {
            "uri"
        };
        
        return Ok(TagInfo {
            uid,
            url: Some(url.clone()),
            text: None,
            language: None,
            record_type: Some(record_type.to_string()),
            wifi_ssid: None,
            wifi_security: None,
            bluetooth_mac: None,
            vcard_name: None,
            vcard_phone: None,
            vcard_email: None,
            vcard_organization: None,
            email_address: if record_type == "email" { Some(url.replace("mailto:", "")) } else { None },
            email_subject: None,
            email_body: None,
            sms_phone: if record_type == "sms" { Some(url.split(':').nth(1).unwrap_or("").to_string()) } else { None },
            sms_message: None,
            phone_number: if record_type == "phone" { Some(url.replace("tel:", "")) } else { None },
        });
    }
    
    // Próbáljuk meg Text-ként olvasni
    if let Ok((text, language)) = ntag.read_ndef_text() {
        return Ok(TagInfo {
            uid,
            url: None,
            text: Some(text),
            language: Some(language),
            record_type: Some("text".to_string()),
            wifi_ssid: None,
            wifi_security: None,
            bluetooth_mac: None,
            vcard_name: None,
            vcard_phone: None,
            vcard_email: None,
            vcard_organization: None,
            email_address: None,
            email_subject: None,
            email_body: None,
            sms_phone: None,
            sms_message: None,
            phone_number: None,
        });
    }
    
    // Próbáljuk meg vCard-ként olvasni
    if let Ok((name, phone, email, org)) = ntag.read_ndef_vcard() {
        return Ok(TagInfo {
            uid,
            url: None,
            text: None,
            language: None,
            record_type: Some("vcard".to_string()),
            wifi_ssid: None,
            wifi_security: None,
            bluetooth_mac: None,
            vcard_name: Some(name),
            vcard_phone: Some(phone),
            vcard_email: Some(email),
            vcard_organization: Some(org),
            email_address: None,
            email_subject: None,
            email_body: None,
            sms_phone: None,
            sms_message: None,
            phone_number: None,
        });
    }
    
    // Ha egyik sem sikerült, csak az UID-t adjuk vissza
    // Próbáljuk meg detektálni a típust
    let detected_type = ntag.detect_ndef_type().unwrap_or_else(|_| "unknown".to_string());
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some(detected_type),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn read_ntag216_url() -> Result<TagInfo, String> {
    read_ntag216().await
}

#[tauri::command]
pub async fn write_wifi_to_ntag216(ssid: String, password: String, security: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    ntag.write_ndef_wifi(&ssid, &password, &security).map_err(|e| {
        format!("Write error: {:?}", e)
    })?;
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some("wifi".to_string()),
        wifi_ssid: Some(ssid),
        wifi_security: Some(security),
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn write_bluetooth_to_ntag216(mac_address: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    ntag.write_ndef_bluetooth(&mac_address).map_err(|e| {
        format!("Write error: {:?}", e)
    })?;
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some("bluetooth".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: Some(mac_address),
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn write_vcard_to_ntag216(name: String, phone: String, email: String, organization: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    ntag.write_ndef_vcard(&name, &phone, &email, &organization).map_err(|e| {
        format!("Write error: {:?}", e)
    })?;
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some("vcard".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: Some(name),
        vcard_phone: Some(phone),
        vcard_email: Some(email),
        vcard_organization: Some(organization),
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn write_email_to_ntag216(email: String, subject: String, body: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    ntag.write_ndef_email(&email, &subject, &body).map_err(|e| {
        format!("Write error: {:?}", e)
    })?;
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some("email".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: Some(email),
        email_subject: Some(subject),
        email_body: Some(body),
        sms_phone: None,
        sms_message: None,
        phone_number: None,
    })
}

#[tauri::command]
pub async fn write_sms_to_ntag216(phone: String, message: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    ntag.write_ndef_sms(&phone, &message).map_err(|e| {
        format!("Write error: {:?}", e)
    })?;
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some("sms".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: Some(phone),
        sms_message: Some(message),
        phone_number: None,
    })
}

#[tauri::command]
pub async fn write_phone_to_ntag216(phone: String) -> Result<TagInfo, String> {
    let reader = NFCReader::new().map_err(|e| format!("NFC Reader error: {:?}", e))?;
    let card = reader
        .wait_for_card(Duration::from_secs(5))
        .map_err(|e| format!("Card detection error: {:?}", e))?;

    let ntag = NTAG216::new(card);
    let uid = ntag.read_uid().map_err(|e| format!("UID read error: {:?}", e))?;
    
    ntag.write_ndef_phone(&phone).map_err(|e| {
        format!("Write error: {:?}", e)
    })?;
    
    Ok(TagInfo {
        uid,
        url: None,
        text: None,
        language: None,
        record_type: Some("phone".to_string()),
        wifi_ssid: None,
        wifi_security: None,
        bluetooth_mac: None,
        vcard_name: None,
        vcard_phone: None,
        vcard_email: None,
        vcard_organization: None,
        email_address: None,
        email_subject: None,
        email_body: None,
        sms_phone: None,
        sms_message: None,
        phone_number: Some(phone),
    })
}

