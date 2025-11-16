use pcsc::Card;

pub struct NTAG216 {
    card: Card,
}

#[derive(Debug)]
pub enum NTAG216Error {
    CardError(String),
    InvalidData,
    WriteError(String),
    ReadError(String),
}

impl NTAG216 {
    pub fn new(card: Card) -> Self {
        NTAG216 { card }
    }

    /// UID olvasása
    pub fn read_uid(&self) -> Result<Vec<u8>, NTAG216Error> {
        // GET UID command: FF CA 00 00 00
        let mut response = vec![0u8; 256];
        self.card
            .transmit(&[0xFF, 0xCA, 0x00, 0x00, 0x00], &mut response)
            .map_err(|e| NTAG216Error::ReadError(format!("UID read error: {}", e)))?;

        // A válasz végén van a status: 0x90 0x00
        // Az UID a válasz elején van, 7 byte
        if response.len() >= 9 && response[response.len() - 2] == 0x90 && response[response.len() - 1] == 0x00 {
            Ok(response[..7].to_vec())
        } else if response.len() >= 7 {
            // Ha nincs status byte, csak az első 7 byte-ot vesszük
            Ok(response[..7].to_vec())
        } else {
            Err(NTAG216Error::ReadError(format!("Invalid UID response: {:?}", &response[..response.len().min(16)])))
        }
    }

    /// Blokk olvasása (4 bytes)
    pub fn read_block(&self, block: u8) -> Result<Vec<u8>, NTAG216Error> {
        // READ BINARY: FF B0 00 [block] [length]
        let mut response = vec![0u8; 256];
        self.card
            .transmit(&[0xFF, 0xB0, 0x00, block, 0x04], &mut response)
            .map_err(|e| NTAG216Error::ReadError(format!("Block read error: {}", e)))?;

        // A válasz végén van a status: 0x90 0x00
        // A blokk adat a válasz elején van, 4 byte
        if response.len() >= 6 && response[response.len() - 2] == 0x90 && response[response.len() - 1] == 0x00 {
            Ok(response[..4].to_vec())
        } else if response.len() >= 4 {
            // Ha nincs status byte, csak az első 4 byte-ot vesszük
            Ok(response[..4].to_vec())
        } else {
            Err(NTAG216Error::ReadError(format!("Invalid block data: {:?}", &response[..response.len().min(8)])))
        }
    }

    /// Blokk írása (4 bytes)
    pub fn write_block(&self, block: u8, data: &[u8]) -> Result<(), NTAG216Error> {
        if data.len() != 4 {
            return Err(NTAG216Error::InvalidData);
        }

        // UPDATE BINARY: FF D6 00 [block] [length] [data...]
        let mut cmd = vec![0xFF, 0xD6, 0x00, block, 0x04];
        cmd.extend_from_slice(data);

        let mut response = vec![0u8; 256];
        self.card
            .transmit(&cmd, &mut response)
            .map_err(|e| NTAG216Error::WriteError(format!("Block write error: {}", e)))?;

        // Check response: should be 90 00 (success)
        // A válasz általában 2 byte: 0x90 0x00
        if response.len() >= 2 && response[0] == 0x90 && response[1] == 0x00 {
            Ok(())
        } else {
            let error_bytes = if response.len() >= 4 { &response[..4] } else { &response[..] };
            Err(NTAG216Error::WriteError(format!("Write failed at block {}: {:?}", block, error_bytes)))
        }
    }

    /// NDEF Text Record létrehozása
    fn create_ndef_text_record(text: &str, language: &str) -> Vec<u8> {
        let mut record = Vec::new();
        let text_bytes = text.as_bytes();
        let language_bytes = language.as_bytes();
        
        // NDEF Record Header
        // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=0x01 (Well Known)
        record.push(0xD1); // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=001
        
        // Type Length (1 byte) - Text type is 1 byte
        record.push(0x01);
        
        // Payload Length (1 byte for SR=1)
        // Payload: status byte (1) + language code + text
        // A status byte tartalmazza a language code hosszát az alsó 6 bitben
        let payload_length = 1 + language_bytes.len() + text_bytes.len();
        record.push(payload_length as u8);
        
        // Type (1 byte) - "T" for Text
        record.push(0x54); // 'T'
        
        // Status byte: bit 7 = UTF-8 (1), bits 6-0 = language code length
        let status_byte = 0x80 | (language_bytes.len() as u8);
        record.push(status_byte);
        
        // Language code
        record.extend_from_slice(language_bytes);
        
        // Text content
        record.extend_from_slice(text_bytes);
        
        eprintln!("Created NDEF Text record: payload_length={}, total_length={}, text='{}', language='{}'", 
                 payload_length, record.len(), text, language);
        
        record
    }

    /// NDEF URI Record létrehozása
    fn create_ndef_uri_record(url: &str) -> Vec<u8> {
        let mut record = Vec::new();

        // URI Prefix meghatározása (NDEF URI Record Type Definition spec)
        // 0x00 = No prefix
        // 0x01 = http://www.
        // 0x02 = https://www.
        // 0x03 = http://
        // 0x04 = https://
        // 0x05 = tel:
        // 0x06 = mailto:
        // 0x07 = ftp://anonymous:anonymous@
        // 0x08 = ftp://ftp.
        // 0x09 = ftps://
        // 0x0A = sftp://
        // 0x0B = smb://
        // 0x0C = nfs://
        // 0x0D = ftp://
        // 0x0E = dav://
        // 0x0F = news:
        // 0x10 = telnet://
        // 0x11 = imap:
        // 0x12 = rtsp://
        // 0x13 = urn:
        // 0x14 = pop:
        // 0x15 = sip:
        // 0x16 = sips:
        // 0x17 = tftp:
        // 0x18 = btspp://
        // 0x19 = btl2cap://
        // 0x1A = btgoep://
        // 0x1B = tcpobex://
        // 0x1C = irdaobex://
        // 0x1D = file://
        // 0x1E = urn:epc:id:
        // 0x1F = urn:epc:tag:
        // 0x20 = urn:epc:pat:
        // 0x21 = urn:epc:raw:
        // 0x22 = urn:epc:
        // 0x23 = urn:nfc:
        let prefix = if url.starts_with("https://www.") {
            (0x02, &url[12..])
        } else if url.starts_with("http://www.") {
            (0x01, &url[11..])
        } else if url.starts_with("https://") {
            (0x04, &url[8..])
        } else if url.starts_with("http://") {
            (0x03, &url[7..])
        } else if url.starts_with("tel:") {
            (0x05, &url[4..]) // tel: prefix
        } else if url.starts_with("mailto:") {
            (0x06, &url[7..]) // mailto: prefix
        } else if url.starts_with("sms:") {
            // SMS URI-knál nincs spec prefix, de használhatjuk a tel: prefix-et a telefonszám részhez
            // Vagy 0x00-t használunk és az egész "sms:..." stringet
            // A legtöbb telefon automatikusan felismeri az "sms:" prefix-et
            (0x00, url) // No prefix, teljes sms: URI
        } else {
            (0x00, url) // No prefix
        };
        
        let url_part_bytes = prefix.1.as_bytes();
        let payload_length = 1 + url_part_bytes.len(); // prefix (1 byte) + URL rész
        
        // NDEF Record Header
        // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=0x01 (Well Known)
        record.push(0xD1); // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=001
        
        // Type Length (1 byte) - URI type is 1 byte
        record.push(0x01);
        
        // Payload Length (1 byte for SR=1)
        // A payload tartalmazza a prefix-et (1 byte) és az URL részt
        record.push(payload_length as u8);
        
        // Type (1 byte) - "U" for URI
        record.push(0x55); // 'U'
        
        // URI Prefix (1 byte)
        record.push(prefix.0);
        
        // URL rész
        record.extend_from_slice(url_part_bytes);
        
        eprintln!("Created NDEF URI record: payload_length={}, total_length={}, url='{}'", payload_length, record.len(), url);
        
        record
    }

    /// NDEF WiFi Simple Configuration Record létrehozása
    fn create_ndef_wifi_record(ssid: &str, password: &str, security: &str) -> Vec<u8> {
        // WiFi Simple Configuration (WSC) - External Type
        // Type: "wfa.org:WFA" (Well Known External Type)
        let type_name = b"wfa.org:WFA";
        
        // WSC Credential TLV struktúra
        let mut credential = Vec::new();
        
        // SSID TLV (0x1045)
        let ssid_bytes = ssid.as_bytes();
        credential.push(0x10);
        credential.push(0x45);
        credential.push((ssid_bytes.len() >> 8) as u8);
        credential.push(ssid_bytes.len() as u8);
        credential.extend_from_slice(ssid_bytes);
        
        // Network Key TLV (0x1027) - jelszó
        let password_bytes = password.as_bytes();
        credential.push(0x10);
        credential.push(0x27);
        credential.push((password_bytes.len() >> 8) as u8);
        credential.push(password_bytes.len() as u8);
        credential.extend_from_slice(password_bytes);
        
        // Authentication Type TLV (0x1003)
        let auth_type = match security.to_lowercase().as_str() {
            "wpa2" | "wpa2-psk" => vec![0x00, 0x20], // WPA2-Personal
            "wpa" | "wpa-psk" => vec![0x00, 0x10],  // WPA-Personal
            "wep" => vec![0x00, 0x08],               // WEP
            "open" | "none" => vec![0x00, 0x01],     // Open
            _ => vec![0x00, 0x20],                   // Default: WPA2
        };
        credential.push(0x10);
        credential.push(0x03);
        credential.push((auth_type.len() >> 8) as u8);
        credential.push(auth_type.len() as u8);
        credential.extend_from_slice(&auth_type);
        
        // Network Key Index TLV (0x1026) - 1 byte, értéke 1
        credential.push(0x10);
        credential.push(0x26);
        credential.push(0x00);
        credential.push(0x01);
        credential.push(0x01);
        
        // Credential TLV (0x100E)
        let mut credential_tlv = Vec::new();
        credential_tlv.push(0x10);
        credential_tlv.push(0x0E);
        credential_tlv.push((credential.len() >> 8) as u8);
        credential_tlv.push(credential.len() as u8);
        credential_tlv.extend_from_slice(&credential);
        
        // NDEF Record létrehozása - External Type (TNF=010 = 0x04)
        let mut record = Vec::new();
        record.push(0xD4); // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=100 (External)
        record.push(type_name.len() as u8);
        record.push(credential_tlv.len() as u8);
        record.extend_from_slice(type_name);
        record.extend_from_slice(&credential_tlv);
        
        eprintln!("Created NDEF WiFi record: SSID='{}', Security='{}'", ssid, security);
        record
    }

    /// NDEF Bluetooth Simple Pairing Record létrehozása
    fn create_ndef_bluetooth_record(mac_address: &str) -> Vec<u8> {
        // Bluetooth MAC cím formátum: "XX:XX:XX:XX:XX:XX"
        let mac_bytes: Vec<u8> = mac_address
            .split(':')
            .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
            .collect();
        
        if mac_bytes.len() != 6 {
            return Vec::new(); // Invalid MAC
        }
        
        // Bluetooth Simple Pairing - External Type
        // Type: "application/vnd.bluetooth.ep.oob"
        let type_name = b"application/vnd.bluetooth.ep.oob";
        
        // MAC cím (6 bytes)
        let mut record = Vec::new();
        record.push(0xD2); // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=010 (MIME)
        record.push(type_name.len() as u8);
        record.push(6); // Payload length
        record.extend_from_slice(type_name);
        record.extend_from_slice(&mac_bytes);
        
        eprintln!("Created NDEF Bluetooth record: MAC='{}'", mac_address);
        record
    }

    /// NDEF vCard Record létrehozása
    fn create_ndef_vcard_record(name: &str, phone: &str, email: &str, organization: &str) -> Vec<u8> {
        // vCard formátum
        let mut vcard = String::new();
        vcard.push_str("BEGIN:VCARD\r\n");
        vcard.push_str("VERSION:3.0\r\n");
        if !name.is_empty() {
            vcard.push_str(&format!("FN:{}\r\n", name));
            vcard.push_str(&format!("N:{}\r\n", name));
        }
        if !phone.is_empty() {
            vcard.push_str(&format!("TEL:{}\r\n", phone));
        }
        if !email.is_empty() {
            vcard.push_str(&format!("EMAIL:{}\r\n", email));
        }
        if !organization.is_empty() {
            vcard.push_str(&format!("ORG:{}\r\n", organization));
        }
        vcard.push_str("END:VCARD\r\n");
        
        let vcard_bytes = vcard.as_bytes();
        let type_name = b"text/vcard";
        
        let mut record = Vec::new();
        record.push(0xD2); // MB=1, ME=1, CF=0, SR=1, IL=0, TNF=010 (MIME)
        record.push(type_name.len() as u8);
        record.push(vcard_bytes.len() as u8);
        record.extend_from_slice(type_name);
        record.extend_from_slice(vcard_bytes);
        
        eprintln!("Created NDEF vCard record: name='{}', phone='{}', email='{}'", name, phone, email);
        record
    }

    /// NDEF Email Record létrehozása (URI formátumban)
    fn create_ndef_email_record(email: &str, subject: &str, body: &str) -> Vec<u8> {
        // Email URI: mailto:email?subject=...&body=...
        let mut email_uri = format!("mailto:{}", email);
        if !subject.is_empty() || !body.is_empty() {
            email_uri.push('?');
            if !subject.is_empty() {
                email_uri.push_str(&format!("subject={}", urlencoding::encode(subject)));
            }
            if !subject.is_empty() && !body.is_empty() {
                email_uri.push('&');
            }
            if !body.is_empty() {
                email_uri.push_str(&format!("body={}", urlencoding::encode(body)));
            }
        }
        
        Self::create_ndef_uri_record(&email_uri)
    }

    /// NDEF SMS Record létrehozása (URI formátumban)
    fn create_ndef_sms_record(phone: &str, message: &str) -> Vec<u8> {
        // SMS URI: sms:+1234567890?body=message
        let sms_uri = if message.is_empty() {
            format!("sms:{}", phone)
        } else {
            format!("sms:{}?body={}", phone, urlencoding::encode(message))
        };
        
        Self::create_ndef_uri_record(&sms_uri)
    }

    /// NDEF Phone Number Record létrehozása (URI formátumban)
    fn create_ndef_phone_record(phone: &str) -> Vec<u8> {
        // Phone URI: tel:+1234567890
        let phone_uri = format!("tel:{}", phone);
        Self::create_ndef_uri_record(&phone_uri)
    }

    /// NDEF Message írása NTAG216-ra (általános)
    pub fn write_ndef_message(&self, ndef_message: Vec<u8>) -> Result<(), NTAG216Error> {
        
        // NTAG216 NDEF kezdőcíme: Block 0x04
        // Először ellenőrizzük a Capability Container-t (Block 0x03)
        // CC: E1 10 12 00 (NTAG216)
        let cc_expected = vec![0xE1, 0x10, 0x12, 0x00];
        let cc_current = self.read_block(0x03)?;
        
        // Ha a CC nem helyes, próbáljuk meg írni
        if cc_current != cc_expected {
            // Próbáljuk meg írni a CC-t
            match self.write_block(0x03, &cc_expected) {
                Ok(_) => {
                    // Ellenőrizzük, hogy sikerült-e az írás
                    let cc_read = self.read_block(0x03)?;
                    if cc_read != cc_expected {
                        // Ha nem sikerült, lehet hogy read-only, de folytatjuk az NDEF írással
                        eprintln!("Warning: Could not write CC block, but continuing with NDEF write");
                    }
                }
                Err(e) => {
                    // Ha az írás nem sikerült, lehet hogy read-only
                    // De folytatjuk az NDEF írással, mert lehet hogy már helyes a CC
                    eprintln!("Warning: Could not write CC block: {:?}, but continuing", e);
                }
            }
        }

        // NDEF Message TLV létrehozása
        let mut tlv_message = Vec::new();
        
        // TLV Tag: 0x03 = NDEF Message
        tlv_message.push(0x03);
        
        // TLV Length
        if ndef_message.len() < 255 {
            tlv_message.push(ndef_message.len() as u8);
        } else {
            tlv_message.push(0xFF);
            tlv_message.push(((ndef_message.len() >> 8) & 0xFF) as u8);
            tlv_message.push((ndef_message.len() & 0xFF) as u8);
        }
        
        // NDEF Message
        tlv_message.extend_from_slice(&ndef_message);
        
        // Terminator TLV: 0xFE
        tlv_message.push(0xFE);

        // Debug: kiírjuk a TLV üzenetet
        eprintln!("TLV message length: {}, content: {:02X?}", tlv_message.len(), &tlv_message[..tlv_message.len().min(32)]);
        
        // Írás blokkonként (4 bytes)
        let mut block_addr = 0x04; // NDEF kezdőcím
        for chunk in tlv_message.chunks(4) {
            let mut block_data = chunk.to_vec();
            block_data.resize(4, 0); // Pad to 4 bytes
            
            eprintln!("Writing block {}: {:02X?}", block_addr, block_data);
            self.write_block(block_addr, &block_data)?;
            
            // Ellenőrizzük, hogy sikerült-e az írás
            let written_data = self.read_block(block_addr)?;
            eprintln!("Read back block {}: {:02X?}", block_addr, written_data);
            if written_data != block_data {
                return Err(NTAG216Error::WriteError(format!(
                    "Write verification failed at block {}: wrote {:?}, read {:?}",
                    block_addr, block_data, written_data
                )));
            }
            
            block_addr += 1;
            
            // NTAG216 max 135 blokk (0x00-0x86)
            if block_addr > 0x86 {
                break;
            }
        }
        
        eprintln!("Successfully wrote {} blocks starting from 0x04", block_addr - 0x04);

        Ok(())
    }

    /// NDEF URI Record írása
    pub fn write_ndef_uri(&self, url: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_uri_record(url);
        self.write_ndef_message(ndef_message)
    }

    /// NDEF Text Record írása
    pub fn write_ndef_text(&self, text: &str, language: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_text_record(text, language);
        self.write_ndef_message(ndef_message)
    }

    /// NDEF Message olvasása NTAG216-ról (általános)
    pub fn read_ndef_message(&self) -> Result<Vec<u8>, NTAG216Error> {
        // Capability Container ellenőrzése (Block 0x03)
        let cc = self.read_block(0x03)?;
        if cc[0] != 0xE1 {
            return Err(NTAG216Error::ReadError("Invalid Capability Container".to_string()));
        }

        // NDEF Message olvasása (Block 0x04-től)
        // Először olvassuk be az összes blokkot egy vektorba
        let mut all_data = Vec::new();
        let mut block_addr = 0x04;
        
        while block_addr <= 0x86 {
            let block = self.read_block(block_addr)?;
            eprintln!("Read block {}: {:02X?}", block_addr, block);
            all_data.extend_from_slice(&block);
            
            // Ha találunk terminator TLV-t (0xFE), megállunk
            if block.contains(&0xFE) {
                eprintln!("Found terminator TLV at block {}", block_addr);
                // A terminator után is olvassunk még egy blokkot, hogy biztosan benne legyen
                // De csak ha még nem értük el a tag végét
                if block_addr < 0x86 {
                    block_addr += 1;
                    let next_block = self.read_block(block_addr)?;
                    all_data.extend_from_slice(&next_block);
                }
                break;
            }
            
            block_addr += 1;
            
            // NTAG216 max 135 blokk (0x00-0x86), NDEF kezdőcíme 0x04
            // Maximum 130 blokkot olvashatunk NDEF adatként (0x04-0x86)
            if block_addr > 0x86 {
                break;
            }
        }
        
        eprintln!("Total data read: {} bytes from blocks 0x04-0x{:02X}", all_data.len(), block_addr - 1);
        
        // TLV struktúra parsing
        let mut ndef_data = Vec::new();
        let mut i = 0;
        
        eprintln!("Parsing TLV structure from {} bytes", all_data.len());
        
        while i < all_data.len() {
            let tag = all_data[i];
            eprintln!("At position {}: tag = 0x{:02X}", i, tag);
            
            if tag == 0xFE {
                // Terminator TLV
                eprintln!("Found terminator TLV");
                break;
            }
            
            if tag == 0x00 {
                // Null TLV, skip
                eprintln!("Skipping null TLV");
                i += 1;
                continue;
            }
            
            if tag == 0x03 {
                // NDEF Message TLV
                eprintln!("Found NDEF Message TLV at position {}", i);
                i += 1;
                if i >= all_data.len() {
                    eprintln!("Error: No length byte after TLV tag");
                    break;
                }
                
                let length_byte = all_data[i];
                eprintln!("TLV length byte: 0x{:02X} ({})", length_byte, length_byte);
                
                let length = if length_byte == 0xFF {
                    // 3-byte length format
                    if i + 2 >= all_data.len() {
                        eprintln!("Error: Not enough bytes for 3-byte length");
                        break;
                    }
                    i += 1;
                    let len = ((all_data[i] as u16) << 8) | (all_data[i + 1] as u16);
                    eprintln!("3-byte length: {}", len);
                    i += 1;
                    len
                } else {
                    length_byte as u16
                };
                
                i += 1;
                eprintln!("TLV length: {}, starting NDEF data at position {}", length, i);
                eprintln!("Available bytes from position {}: {}", i, all_data.len() - i);
                
                // NDEF Message adatok olvasása
                let end_pos = i + length as usize;
                if end_pos <= all_data.len() {
                    ndef_data = all_data[i..end_pos].to_vec();
                    eprintln!("Extracted {} bytes of NDEF data", ndef_data.len());
                    break;
                } else {
                    // Ha nincs elég adat, folytassuk az olvasást
                    eprintln!("Warning: TLV length {} exceeds available data ({} bytes), continuing to read more blocks", length, all_data.len() - i);
                    
                    // Folytatjuk az olvasást, amíg meg nem találjuk a terminator TLV-t vagy el nem érjük a tag végét
                    let mut continue_block = block_addr;
                    while continue_block <= 0x86 && all_data.len() < end_pos {
                        continue_block += 1;
                        if continue_block > 0x86 {
                            break;
                        }
                        let next_block = self.read_block(continue_block)?;
                        eprintln!("Read additional block {}: {:02X?}", continue_block, next_block);
                        all_data.extend_from_slice(&next_block);
                        
                        // Ha találunk terminator TLV-t, megállunk
                        if next_block.contains(&0xFE) {
                            eprintln!("Found terminator TLV at block {}", continue_block);
                            break;
                        }
                    }
                    
                    // Most próbáljuk meg újra kivenni az NDEF adatokat
                    if end_pos <= all_data.len() {
                        ndef_data = all_data[i..end_pos].to_vec();
                        eprintln!("Extracted {} bytes of NDEF data after reading more blocks", ndef_data.len());
                    } else {
                        // Ha még mindig nincs elég, használjuk az elérhető adatot
                        eprintln!("Still insufficient data, using available {} bytes", all_data.len() - i);
                        ndef_data = all_data[i..].to_vec();
                    }
                    break;
                }
            } else {
                // Ismeretlen TLV, skip
                eprintln!("Skipping unknown TLV tag 0x{:02X}", tag);
                i += 1;
                if i >= all_data.len() {
                    break;
                }
                let length = all_data[i] as usize;
                eprintln!("Unknown TLV length: {}, skipping", length);
                i += 1 + length;
            }
        }

        if ndef_data.is_empty() {
            return Err(NTAG216Error::ReadError("No NDEF message found".to_string()));
        }

        // Debug: kiírjuk az NDEF adatokat
        eprintln!("NDEF data length: {}, content: {:02X?}", ndef_data.len(), &ndef_data[..ndef_data.len().min(32)]);
        
        // NDEF Record parsing
        if ndef_data.len() < 4 {
            return Err(NTAG216Error::ReadError(format!("Invalid NDEF record: too short ({} bytes)", ndef_data.len())));
        }

        let _header = ndef_data[0]; // NDEF header (MB, ME, CF, SR, IL, TNF flags)
        let type_length = ndef_data[1] as usize;
        let payload_length = ndef_data[2] as usize;
        
        eprintln!("NDEF header: 0x{:02X}, type_length: {}, payload_length: {}", _header, type_length, payload_length);
        eprintln!("Required length: {}, actual length: {}", 3 + type_length + payload_length, ndef_data.len());
        
        if ndef_data.len() < 3 + type_length + payload_length {
            return Err(NTAG216Error::ReadError(format!(
                "Invalid NDEF record length: need {} bytes, have {} bytes (type_len: {}, payload_len: {})",
                3 + type_length + payload_length, ndef_data.len(), type_length, payload_length
            )));
        }

        let type_start = 3;
        let payload_start = type_start + type_length;
        
        // Visszaadjuk az NDEF adatokat
        Ok(ndef_data)
    }

    /// NDEF URI Record olvasása
    pub fn read_ndef_uri(&self) -> Result<String, NTAG216Error> {
        let ndef_data = self.read_ndef_message()?;
        
        if ndef_data.len() < 4 {
            return Err(NTAG216Error::ReadError("Invalid NDEF record".to_string()));
        }

        let type_length = ndef_data[1] as usize;
        let payload_length = ndef_data[2] as usize;
        let type_start = 3;
        let payload_start = type_start + type_length;
        
        // URI Record ellenőrzése (type should be 'U' = 0x55)
        if ndef_data[type_start] != 0x55 {
            return Err(NTAG216Error::ReadError("Not a URI record".to_string()));
        }

        // URI prefix és URL
        let prefix_code = ndef_data[payload_start];
        let url_part = &ndef_data[payload_start + 1..payload_start + payload_length];
        let url_part_str = String::from_utf8_lossy(url_part);

        let url = match prefix_code {
            0x01 => format!("http://www.{}", url_part_str),
            0x02 => format!("https://www.{}", url_part_str),
            0x03 => format!("http://{}", url_part_str),
            0x04 => format!("https://{}", url_part_str),
            0x05 => format!("tel:{}", url_part_str), // tel: prefix
            0x06 => format!("mailto:{}", url_part_str), // mailto: prefix
            _ => {
                // Ha nincs prefix és nem tartalmazza a scheme-t, akkor lehet hogy teljes URI
                if url_part_str.starts_with("sms:") || url_part_str.starts_with("tel:") || url_part_str.starts_with("mailto:") {
                    url_part_str.to_string()
                } else {
                    url_part_str.to_string()
                }
            },
        };

        Ok(url)
    }

    /// NDEF Text Record olvasása
    pub fn read_ndef_text(&self) -> Result<(String, String), NTAG216Error> {
        let ndef_data = self.read_ndef_message()?;
        
        if ndef_data.len() < 4 {
            return Err(NTAG216Error::ReadError("Invalid NDEF record".to_string()));
        }

        let type_length = ndef_data[1] as usize;
        let payload_length = ndef_data[2] as usize;
        let type_start = 3;
        let payload_start = type_start + type_length;
        
        // Text Record ellenőrzése (type should be 'T' = 0x54)
        if ndef_data[type_start] != 0x54 {
            return Err(NTAG216Error::ReadError("Not a Text record".to_string()));
        }

        // Status byte: bit 7 = UTF-8 flag, bits 6-0 = language code length
        let status_byte = ndef_data[payload_start];
        let language_length = (status_byte & 0x3F) as usize;
        
        if payload_length < 1 + language_length {
            return Err(NTAG216Error::ReadError("Invalid Text record payload".to_string()));
        }

        // Language code
        let language_start = payload_start + 1;
        let language = String::from_utf8_lossy(
            &ndef_data[language_start..language_start + language_length]
        ).to_string();

        // Text content
        let text_start = language_start + language_length;
        let text = String::from_utf8_lossy(
            &ndef_data[text_start..payload_start + payload_length]
        ).to_string();

        Ok((text, language))
    }

    /// NDEF vCard Record olvasása
    pub fn read_ndef_vcard(&self) -> Result<(String, String, String, String), NTAG216Error> {
        let ndef_data = self.read_ndef_message()?;
        
        if ndef_data.len() < 4 {
            return Err(NTAG216Error::ReadError("Invalid NDEF record".to_string()));
        }

        let type_length = ndef_data[1] as usize;
        let payload_length = ndef_data[2] as usize;
        let type_start = 3;
        let payload_start = type_start + type_length;
        
        // vCard Record ellenőrzése (MIME type should be "text/vcard")
        let type_name = String::from_utf8_lossy(
            &ndef_data[type_start..type_start + type_length]
        ).to_string();
        
        if type_name != "text/vcard" {
            return Err(NTAG216Error::ReadError("Not a vCard record".to_string()));
        }

        // vCard payload olvasása
        let vcard_data = &ndef_data[payload_start..payload_start + payload_length];
        let vcard_text = String::from_utf8_lossy(vcard_data).to_string();
        
        // vCard parsing
        let mut name = String::new();
        let mut phone = String::new();
        let mut email = String::new();
        let mut organization = String::new();
        
        for line in vcard_text.lines() {
            let line = line.trim();
            if line.starts_with("FN:") {
                name = line[3..].trim().to_string();
            } else if line.starts_with("N:") {
                if name.is_empty() {
                    name = line[2..].trim().to_string();
                }
            } else if line.starts_with("TEL:") {
                phone = line[4..].trim().to_string();
            } else if line.starts_with("EMAIL:") {
                email = line[6..].trim().to_string();
            } else if line.starts_with("ORG:") {
                organization = line[4..].trim().to_string();
            }
        }
        
        Ok((name, phone, email, organization))
    }

    /// NDEF rekord típusának meghatározása
    pub fn detect_ndef_type(&self) -> Result<String, NTAG216Error> {
        let ndef_data = self.read_ndef_message()?;
        
        if ndef_data.len() < 4 {
            return Err(NTAG216Error::ReadError("Invalid NDEF record".to_string()));
        }

        let header = ndef_data[0];
        let type_length = ndef_data[1] as usize;
        let type_start = 3;
        
        // TNF meghatározása (első 3 bit)
        let tnf = (header & 0x07) as u8;
        
        if tnf == 0x01 && type_length == 1 {
            // Well Known Type
            match ndef_data[type_start] {
                0x55 => Ok("uri".to_string()),      // 'U' - URI
                0x54 => Ok("text".to_string()),     // 'T' - Text
                _ => Ok("unknown".to_string()),
            }
        } else if tnf == 0x04 {
            // External Type
            let type_name = String::from_utf8_lossy(
                &ndef_data[type_start..type_start + type_length]
            ).to_string();
            
            if type_name == "wfa.org:WFA" {
                Ok("wifi".to_string())
            } else if type_name == "application/vnd.bluetooth.ep.oob" {
                Ok("bluetooth".to_string())
            } else {
                Ok("unknown".to_string())
            }
        } else if tnf == 0x02 {
            // MIME Type
            let type_name = String::from_utf8_lossy(
                &ndef_data[type_start..type_start + type_length]
            ).to_string();
            
            if type_name == "text/vcard" {
                Ok("vcard".to_string())
            } else {
                Ok("unknown".to_string())
            }
        } else {
            // URI alapú típusok (mailto, sms, tel)
            if let Ok(url) = self.read_ndef_uri() {
                if url.starts_with("mailto:") {
                    return Ok("email".to_string());
                } else if url.starts_with("sms:") {
                    return Ok("sms".to_string());
                } else if url.starts_with("tel:") {
                    return Ok("phone".to_string());
                }
            }
            Ok("unknown".to_string())
        }
    }

    /// NDEF WiFi Record írása
    pub fn write_ndef_wifi(&self, ssid: &str, password: &str, security: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_wifi_record(ssid, password, security);
        self.write_ndef_message(ndef_message)
    }

    /// NDEF Bluetooth Record írása
    pub fn write_ndef_bluetooth(&self, mac_address: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_bluetooth_record(mac_address);
        if ndef_message.is_empty() {
            return Err(NTAG216Error::WriteError("Invalid MAC address format. Use format: XX:XX:XX:XX:XX:XX".to_string()));
        }
        self.write_ndef_message(ndef_message)
    }

    /// NDEF vCard Record írása
    pub fn write_ndef_vcard(&self, name: &str, phone: &str, email: &str, organization: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_vcard_record(name, phone, email, organization);
        self.write_ndef_message(ndef_message)
    }

    /// NDEF Email Record írása
    pub fn write_ndef_email(&self, email: &str, subject: &str, body: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_email_record(email, subject, body);
        self.write_ndef_message(ndef_message)
    }

    /// NDEF SMS Record írása
    pub fn write_ndef_sms(&self, phone: &str, message: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_sms_record(phone, message);
        self.write_ndef_message(ndef_message)
    }

    /// NDEF Phone Record írása
    pub fn write_ndef_phone(&self, phone: &str) -> Result<(), NTAG216Error> {
        let ndef_message = Self::create_ndef_phone_record(phone);
        self.write_ndef_message(ndef_message)
    }
}

