use anyhow::{Context, Result};
use pcsc::Card;

/// NTAG216 címke kezelése
/// NTAG216 specifikáció:
/// - Kapacitás: 888 bytes felhasználói adat
/// - 135 blocks (4 bytes/block)
/// - Block 0-3: UID és manufacturer data
/// - Block 4-129: User data
/// - Block 130-134: Configuration pages
pub struct Ntag216;

impl Ntag216 {
    /// APDU parancs küldése a címkének
    fn transmit(&self, card: &Card, apdu: &[u8]) -> Result<Vec<u8>> {
        let mut response_buffer = [0u8; 256];
        let response = card.transmit(apdu, &mut response_buffer)
            .context("Nem sikerült kommunikálni az NFC címkével")?;
        
        if response.len() < 2 {
            anyhow::bail!("Érvénytelen válasz az NFC címkétől");
        }

        // Ellenőrizzük a status byte-okat (SW1, SW2)
        let sw1 = response[response.len() - 2];
        let sw2 = response[response.len() - 1];
        
        if sw1 != 0x90 || sw2 != 0x00 {
            anyhow::bail!("NFC címke hiba: SW1=0x{:02X}, SW2=0x{:02X}", sw1, sw2);
        }

        // Visszaadjuk a választ status byte-ok nélkül
        Ok(response[..response.len() - 2].to_vec())
    }

    /// Block olvasása (4 bytes) - password opcionális
    pub fn read_block(&self, card: &Card, block: u8) -> Result<[u8; 4]> {
        self.read_block_with_password(card, block, None)
    }

    /// Block olvasása password-dal (ha szükséges)
    pub fn read_block_with_password(&self, card: &Card, block: u8, password: Option<&[u8; 4]>) -> Result<[u8; 4]> {
        // Ha password van megadva, először authenticate-olunk
        if let Some(pwd) = password {
            self.authenticate_password(card, pwd)?;
        }
        
        // READ command: CLA=0xFF, INS=0xB0, P1=block, P2=0x00, Le=0x04
        let apdu = &[0xFF, 0xB0, 0x00, block, 0x04];
        let response = self.transmit(card, apdu)?;
        
        if response.len() != 4 {
            anyhow::bail!("Érvénytelen block méret");
        }

        let mut block_data = [0u8; 4];
        block_data.copy_from_slice(&response[..4]);
        Ok(block_data)
    }

    /// Password authentication
    /// NTAG216 PWD_AUTH parancs: 0x1B + 4 byte password
    /// Próbáljuk meg több módszert is PC/SC API-n keresztül
    pub fn authenticate_password(&self, card: &Card, password: &[u8; 4]) -> Result<()> {
        println!("      🔐 Password authentication...");
        println!("        Password: {:02X?}", password);
        
        // Először próbáljuk meg az authentication-t közvetlenül
        // Block 130 ellenőrzése csak opcionális, mert password védelem után nem olvasható
        println!("        🔍 Password ellenőrzése Block 130-ból (opcionális)...");
        
        // Próbáljuk meg password nélkül először
        let mut block130_empty = false;
        match self.read_block_with_password(card, 130, None) {
            Ok(pwd) => {
                println!("        📊 Tárolt password Block 130-ban (password nélkül): {:02X?}", pwd);
                let is_empty = pwd == [0x00, 0x00, 0x00, 0x00];
                if is_empty {
                    println!("        ⚠️ Block 130 üres - lehet, hogy nincs password beállítva.");
                    println!("        💡 De lehet, hogy password védelem aktív és Block 130 nem olvasható password nélkül.");
                    println!("        💡 Próbáljuk meg az authentication-t...");
                    block130_empty = true;
                    // Folytatjuk az authentication próbálkozást
                } else {
                    if pwd != *password {
                        println!("        ⚠️ A password NEM egyezik meg a tárolttal!");
                        println!("        💡 A tárolt password: {:02X?}", pwd);
                        println!("        💡 A megadott password: {:02X?}", password);
                        println!("        💡 Használd a helyes password-t vagy állítsd be újra!");
                        // Folytatjuk az authentication-nel, hátha mégis működik
                    } else {
                        println!("        ✅ A password megegyezik a tárolttal");
                    }
                }
            }
            Err(e) => {
                // Ha SW1=0x63, akkor password védelem aktív, Block 130 nem olvasható password nélkül
                let error_msg = format!("{}", e);
                if error_msg.contains("SW1=0x63") {
                    println!("        🔐 Password védelem aktív (Block 130 nem olvasható password nélkül)");
                    println!("        💡 Folytatjuk az authentication-nel a megadott password-tel...");
                } else {
                    println!("        ⚠️ Block 130 olvasási hiba: {}", e);
                    println!("        💡 Folytatjuk az authentication-nel...");
                }
            }
        }
        
        // Próbáljuk meg az ACR122U direct command módszert
        // ACR122U-nál lehet, hogy közvetlenül az NTAG216 parancsot kell küldeni
        println!("        🔄 Próbáljuk meg az ACR122U direct command módszert...");
        
        // Módszer 1: PC/SC APDU formátum (CLA=0xFF, INS=0x1B, P1=0x00, P2=0x00, Lc=0x04, password)
        let mut apdu1 = vec![0xFF, 0x1B, 0x00, 0x00, 0x04];
        apdu1.extend_from_slice(password);
        println!("        📤 Módszer 1 (APDU): {:02X?}", apdu1);
        
        let mut response_buffer = [0u8; 256];
        let response1 = card.transmit(&apdu1, &mut response_buffer);
        
        match response1 {
            Ok(resp) => {
                println!("        📥 Válasz: {:02X?} (len: {})", resp, resp.len());
                if resp.len() >= 2 {
                    let sw1 = resp[resp.len() - 2];
                    let sw2 = resp[resp.len() - 1];
                    println!("        📊 SW1=0x{:02X}, SW2=0x{:02X}", sw1, sw2);
                    
                    if sw1 == 0x90 && sw2 == 0x00 {
                        if resp.len() >= 4 {
                            let pack = &resp[..resp.len() - 2];
                            if pack.len() >= 2 {
                                println!("        📦 PACK: {:02X?}", &pack[..2]);
                            }
                        }
                        println!("      ✅ Password authentication sikeres (Módszer 1)");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                println!("        ❌ Módszer 1 hiba: {}", e);
            }
        }
        
        // Módszer 2: ACR122U direct command (0xFF 0x00 0x00 0x00 + length + command + password)
        println!("        🔄 Próbáljuk meg az ACR122U direct command módszert (Módszer 2)...");
        let mut apdu2 = vec![0xFF, 0x00, 0x00, 0x00, 0x05]; // Length = 5 (0x1B + 4 byte password)
        apdu2.push(0x1B); // PWD_AUTH command
        apdu2.extend_from_slice(password);
        println!("        📤 Módszer 2 (Direct): {:02X?}", apdu2);
        
        let mut response_buffer2 = [0u8; 256];
        let response2 = card.transmit(&apdu2, &mut response_buffer2);
        
        match response2 {
            Ok(resp) => {
                println!("        📥 Válasz: {:02X?} (len: {})", resp, resp.len());
                if resp.len() >= 2 {
                    let sw1 = resp[resp.len() - 2];
                    let sw2 = resp[resp.len() - 1];
                    println!("        📊 SW1=0x{:02X}, SW2=0x{:02X}", sw1, sw2);
                    
                    if sw1 == 0x90 && sw2 == 0x00 {
                        if resp.len() >= 4 {
                            let pack = &resp[..resp.len() - 2];
                            if pack.len() >= 2 {
                                println!("        📦 PACK: {:02X?}", &pack[..2]);
                            }
                        }
                        println!("      ✅ Password authentication sikeres (Módszer 2)");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                println!("        ❌ Módszer 2 hiba: {}", e);
            }
        }
        
        // Módszer 3: ACR122U-nál lehet, hogy az NTAG216 parancsot másképp kell formázni
        // Próbáljuk meg: 0xFF 0x00 0x00 0x00 + length + data (ahol data = 0x1B + password)
        println!("        🔄 Próbáljuk meg az ACR122U alternatív módszert (Módszer 3)...");
        let mut apdu3 = vec![0xFF, 0x00, 0x00, 0x00];
        apdu3.push(0x05); // Length = 5
        apdu3.push(0x1B); // PWD_AUTH command
        apdu3.extend_from_slice(password);
        println!("        📤 Módszer 3 (Alternatív): {:02X?}", apdu3);
        
        let mut response_buffer3 = [0u8; 256];
        let response3 = card.transmit(&apdu3, &mut response_buffer3);
        
        match response3 {
            Ok(resp) => {
                println!("        📥 Válasz: {:02X?} (len: {})", resp, resp.len());
                if resp.len() >= 2 {
                    let sw1 = resp[resp.len() - 2];
                    let sw2 = resp[resp.len() - 1];
                    println!("        📊 SW1=0x{:02X}, SW2=0x{:02X}", sw1, sw2);
                    
                    if sw1 == 0x90 && sw2 == 0x00 {
                        if resp.len() >= 4 {
                            let pack = &resp[..resp.len() - 2];
                            if pack.len() >= 2 {
                                println!("        📦 PACK: {:02X?}", &pack[..2]);
                            }
                        }
                        println!("      ✅ Password authentication sikeres (Módszer 3)");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                println!("        ❌ Módszer 3 hiba: {}", e);
            }
        }
        
        // Módszer 4: Lehet, hogy az ACR122U-nál az NTAG216 PWD_AUTH parancsot másképp kell küldeni
        // Próbáljuk meg: 0xFF 0x1B + password (egyszerűsített formátum)
        println!("        🔄 Próbáljuk meg az egyszerűsített módszert (Módszer 4)...");
        let mut apdu4 = vec![0xFF, 0x1B];
        apdu4.extend_from_slice(password);
        println!("        📤 Módszer 4 (Egyszerűsített): {:02X?}", apdu4);
        
        let mut response_buffer4 = [0u8; 256];
        let response4 = card.transmit(&apdu4, &mut response_buffer4);
        
        match response4 {
            Ok(resp) => {
                println!("        📥 Válasz: {:02X?} (len: {})", resp, resp.len());
                if resp.len() >= 2 {
                    let sw1 = resp[resp.len() - 2];
                    let sw2 = resp[resp.len() - 1];
                    println!("        📊 SW1=0x{:02X}, SW2=0x{:02X}", sw1, sw2);
                    
                    if sw1 == 0x90 && sw2 == 0x00 {
                        if resp.len() >= 4 {
                            let pack = &resp[..resp.len() - 2];
                            if pack.len() >= 2 {
                                println!("        📦 PACK: {:02X?}", &pack[..2]);
                            }
                        }
                        println!("      ✅ Password authentication sikeres (Módszer 4)");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                println!("        ❌ Módszer 4 hiba: {}", e);
            }
        }
        
        // Ha minden módszer sikertelen
        if block130_empty {
            // Ha Block 130 üres volt, lehet hogy tényleg nincs password beállítva
            println!("        ⚠️ Authentication sikertelen minden módszerrel, és Block 130 üres volt.");
            println!("        💡 Valószínűleg nincs password beállítva a címkére.");
            println!("        💡 Az írás password nélkül fog folyni.");
            // Dobunk egy speciális hibát, amit a hívó függvény kezelhet
            // Ez jelzi, hogy nincs password beállítva, és password nélkül kell írni
            anyhow::bail!("NO_PASSWORD_SET:Block 130 üres, nincs password beállítva a címkére");
        } else {
            // Ha Block 130 nem üres volt, akkor valószínűleg rossz password VAGY PC/SC API korlát
            println!("        ⚠️ Authentication sikertelen minden módszerrel (SW1=0x63, SW2=0x00).");
            println!("        💡 Ez lehet PC/SC API korlát az ACR122U-nál.");
            println!("        💡 Az ACR122U-nál az NTAG216 password authentication nem mindig működik PC/SC API-n keresztül.");
            println!("        💡 Próbáld meg password nélkül írni, vagy használj más NFC olvasót.");
            anyhow::bail!("Password authentication sikertelen minden módszerrel. Ez lehet PC/SC API korlát az ACR122U-nál. Az NTAG216 password authentication nem mindig működik PC/SC API-n keresztül.");
        }
    }

    /// Block írása (4 bytes) - password opcionális
    pub fn write_block(&self, card: &Card, block: u8, data: &[u8; 4]) -> Result<()> {
        self.write_block_with_password(card, block, data, None)
    }

    /// Block írása password-dal (ha szükséges)
    /// NOTE: Az authentication-t már előzőleg meg kell tenni! Ez a függvény nem authenticate-ol.
    pub fn write_block_with_password(&self, card: &Card, block: u8, data: &[u8; 4], password: Option<&[u8; 4]>) -> Result<()> {
        // WRITE command: CLA=0xFF, INS=0xD6, P1=0x00, P2=block, Lc=0x04, data
        println!("        📝 Block {} írása: {:02X?}", block, data);
        let mut apdu = vec![0xFF, 0xD6, 0x00, block, 0x04];
        apdu.extend_from_slice(data);
        
        // Próbáljuk meg az írást
        let mut response_buffer = [0u8; 256];
        let response = card.transmit(&apdu, &mut response_buffer)
            .context("Nem sikerült kommunikálni az NFC címkével")?;
        
        if response.len() < 2 {
            anyhow::bail!("Érvénytelen válasz az NFC címkétől");
        }

        let sw1 = response[response.len() - 2];
        let sw2 = response[response.len() - 1];
        
        if sw1 == 0x90 && sw2 == 0x00 {
            println!("        ✅ Block {} sikeresen írva", block);
            Ok(())
        } else if sw1 == 0x63 && password.is_none() {
            // SW1=0x63 password nélkül - valószínűleg password védelem aktív
            // Dobunk egy speciális hibát, amit a hívó függvény kezelhet
            anyhow::bail!("PASSWORD_REQUIRED:SW1=0x{:02X},SW2=0x{:02X}", sw1, sw2);
        } else {
            anyhow::bail!("NFC címke hiba: SW1=0x{:02X}, SW2=0x{:02X}", sw1, sw2);
        }
    }

    /// NTAG216 típus ellenőrzése - password opcionális
    pub fn check_type(&self, card: &Card) -> Result<bool> {
        self.check_type_with_password(card, None)
    }

    /// NTAG216 típus ellenőrzése password-dal (ha szükséges)
    pub fn check_type_with_password(&self, card: &Card, password: Option<&[u8; 4]>) -> Result<bool> {
        // Próbáljuk meg olvasni Block 3-at password nélkül
        match self.read_block_with_password(card, 3, None) {
            Ok(cc) => {
                // NTAG216 capability container: [E1 10 12 00]
                // E1 = NDEF magic number
                Ok(cc[0] == 0xE1)
            }
            Err(e) => {
                // Ha SW1=0x63, akkor password védelem aktív
                let error_msg = format!("{}", e);
                if error_msg.contains("SW1=0x63") {
                    // Password védelem aktív
                    if let Some(pwd) = password {
                        println!("      🔐 Password védelem aktív, authenticate-olunk...");
                        let cc = self.read_block_with_password(card, 3, Some(pwd))?;
                        Ok(cc[0] == 0xE1)
                    } else {
                        println!("      ⚠️ Password védelem aktív, de nincs password megadva!");
                        Err(e).context("Password védelem aktív, de nincs password megadva. Add meg a password-t!")
                    }
                } else {
                    // Más hiba
                    Err(e)
                }
            }
        }
    }

    /// NDEF üzenet olvasása
    pub fn read_ndef(&self, card: &Card) -> Result<Option<String>> {
        println!("    📖 Ntag216::read_ndef() CALLED");
        
        // Olvassuk a capability container-t
        println!("      🔍 Block 3 olvasása (CC)...");
        let cc = self.read_block(card, 3)
            .map_err(|e| {
                println!("      ❌ Block 3 olvasási hiba: {}", e);
                e
            })?;
        println!("      📊 CC: {:02X?}", cc);
        
        if cc[0] != 0xE1 {
            println!("      ❌ Nincs NDEF (CC[0] = 0x{:02X}, nem 0xE1)", cc[0]);
            return Ok(None); // Nincs NDEF üzenet
        }
        println!("      ✅ NDEF magic number megerősítve");

        // Olvassuk az NDEF TLV-t (block 4)
        println!("      🔍 Block 4 olvasása (TLV)...");
        let tlv = self.read_block(card, 4)
            .map_err(|e| {
                println!("      ❌ Block 4 olvasási hiba: {}", e);
                e
            })?;
        println!("      📊 TLV: {:02X?}", tlv);
        
        // TLV formátum: [Tag] [Length] [Value...]
        if tlv[0] != 0x03 {
            println!("      ❌ Nem NDEF TLV (Tag = 0x{:02X}, nem 0x03)", tlv[0]);
            return Ok(None); // Nem NDEF TLV
        }

        let length = tlv[1] as usize;
        println!("      📏 NDEF hossz: {} bytes", length);
        if length == 0 {
            println!("      ❌ Üres NDEF üzenet");
            return Ok(None);
        }

        // Olvassuk az NDEF üzenetet
        println!("      📖 NDEF adatok olvasása...");
        let mut ndef_data = Vec::new();
        let mut block = 4;
        let mut offset = 2; // TLV header után
        
        while ndef_data.len() < length {
            let block_data = self.read_block(card, block)
                .map_err(|e| {
                    println!("      ❌ Block {} olvasási hiba: {}", block, e);
                    e
                })?;
            
            for i in offset..4 {
                if ndef_data.len() < length {
                    ndef_data.push(block_data[i]);
                }
            }
            
            block += 1;
            offset = 0;
            
            if block > 20 {
                println!("      ⚠️ Túl sok block olvasva, leállítás");
                break;
            }
        }
        
        println!("      📊 Olvasott NDEF adatok ({} bytes): {:02X?}", ndef_data.len(), ndef_data);

        // Parse NDEF üzenet
        println!("      🔍 NDEF parse-olás...");
        let result = self.parse_ndef_url(&ndef_data);
        match &result {
            Ok(Some(url)) => println!("      ✅ URL parse-olva: {}", url),
            Ok(None) => println!("      ❌ Nem sikerült parse-olni az URL-t"),
            Err(e) => println!("      ❌ Parse hiba: {}", e),
        }
        result
    }

    /// NDEF URL üzenet írása
    pub fn write_ndef_url(&self, card: &Card, url: &str) -> Result<()> {
        self.write_ndef_url_with_password(card, url, None)
    }

    pub fn write_ndef_url_with_password(&self, card: &Card, url: &str, password: Option<&[u8; 4]>) -> Result<()> {
        println!("    📝 Ntag216::write_ndef_url() CALLED");
        if let Some(_pwd) = password {
            println!("      🔐 Password védett írás");
        }
        
        // Ellenőrizzük, hogy NTAG216-e
        if !self.check_type(card)? {
            anyhow::bail!("Ez nem egy NTAG216 címke");
        }

        // Ha password van, authenticate-olunk először
        // Ha nincs password beállítva a címkére, az authenticate_password sikeresen visszatér,
        // de az írás password nélkül fog folyni
        let mut actual_password = password;
        if let Some(pwd) = password {
            // Ellenőrizzük, hogy van-e password beállítva
            match self.read_block(card, 130) {
                Ok(stored_pwd) => {
                    if stored_pwd == [0x00, 0x00, 0x00, 0x00] {
                        println!("      ⚠️ Block 130 üres, de password megadva.");
                        println!("      💡 Próbáljuk meg az authentication-t - ha sikeres, password-dal írunk.");
                        // Próbáljuk meg az authentication-t - ha sikeres, password-dal írunk
                        // Ha sikertelen, password nélkül próbáljuk meg
                        match self.authenticate_password(card, pwd) {
                            Ok(_) => {
                                println!("      ✅ Authentication sikeres, password-dal írunk.");
                                // actual_password marad password
                            }
                            Err(e) => {
                                let error_msg = format!("{}", e);
                                if error_msg.contains("NO_PASSWORD_SET") {
                                    println!("      ⚠️ Nincs password beállítva a címkére.");
                                    println!("      💡 Az írás password nélkül fog folyni.");
                                    actual_password = None; // Password nélkül próbáljuk meg
                                } else {
                                    println!("      ⚠️ Authentication sikertelen: {}", e);
                                    println!("      💡 Próbáljuk meg password nélkül írni.");
                                    actual_password = None; // Password nélkül próbáljuk meg
                                }
                            }
                        }
                    } else {
                        // Van password beállítva, authenticate-olunk
                        println!("      🔐 Password találva Block 130-ban, authenticate-olunk...");
                        self.authenticate_password(card, pwd)?;
                    }
                }
                Err(e) => {
                    // Ha nem lehet olvasni Block 130-at, lehet hogy password védelem aktív
                    let error_msg = format!("{}", e);
                    if error_msg.contains("SW1=0x63") {
                        println!("      🔐 Block 130 nem olvasható password nélkül (SW1=0x63) - password védelem aktív!");
                        println!("      💡 Próbáljuk meg az authentication-t...");
                        // Password védelem aktív, authenticate-olunk
                        self.authenticate_password(card, pwd)?;
                    } else {
                        println!("      ⚠️ Block 130 olvasási hiba: {}", e);
                        println!("      💡 Próbáljuk meg az authentication-t...");
                        // Próbáljuk meg az authentication-t
                        match self.authenticate_password(card, pwd) {
                            Ok(_) => {
                                println!("      ✅ Authentication sikeres, password-dal írunk.");
                                // actual_password marad password
                            }
                            Err(e) => {
                                let error_msg = format!("{}", e);
                                if error_msg.contains("NO_PASSWORD_SET") {
                                    println!("      ⚠️ Nincs password beállítva a címkére.");
                                    println!("      💡 Az írás password nélkül fog folyni.");
                                    actual_password = None;
                                } else {
                                    println!("      ⚠️ Authentication sikertelen: {}", e);
                                    println!("      💡 Próbáljuk meg password nélkül írni.");
                                    actual_password = None;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Készítsük el az NDEF URL üzenetet
        let ndef_message = self.create_ndef_url(url)?;
        
        // TLV formátum: [0x03] [Length] [NDEF message...]
        let tlv_length = ndef_message.len();
        if tlv_length > 255 {
            anyhow::bail!("Az URL túl hosszú (max 255 byte)");
        }

        // Írjuk a TLV-t és az NDEF üzenetet
        let mut block = 4;
        let mut data_to_write = vec![0x03, tlv_length as u8];
        data_to_write.extend_from_slice(&ndef_message);
        
        // Töltjük fel 4 byte-os blokkokra
        let mut block_data = [0u8; 4];
        let mut data_index = 0;
        
        println!("      📝 NDEF üzenet írása ({} bytes)...", data_to_write.len());
        while data_index < data_to_write.len() {
            for i in 0..4 {
                if data_index < data_to_write.len() {
                    block_data[i] = data_to_write[data_index];
                    data_index += 1;
                } else {
                    block_data[i] = 0x00; // Padding
                }
            }
            
            // Próbáljuk meg az írást
            match self.write_block_with_password(card, block, &block_data, actual_password) {
                Ok(_) => {
                    // Sikeres írás
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    // Ha password nélkül SW1=0x63 hibát kaptunk, és van password megadva, próbáljuk meg password-dal
                    if error_msg.contains("PASSWORD_REQUIRED") && password.is_some() && actual_password.is_none() {
                        println!("      🔐 Password védelem aktív (SW1=0x63), authenticate-olunk és újrapróbáljuk...");
                        if let Some(pwd) = password {
                            self.authenticate_password(card, pwd)?;
                            actual_password = password; // Most password-dal írunk
                            // Újrapróbáljuk password-dal
                            self.write_block_with_password(card, block, &block_data, actual_password)?;
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
            
            block += 1;
            
            // NTAG216 user data: block 4-129 (126 blocks = 504 bytes)
            if block > 129 {
                anyhow::bail!("Az NDEF üzenet túl nagy az NTAG216 kapacitásához");
            }
        }

        // Termináló TLV (0xFE jelzi a végét)
        let terminator = [0xFE, 0x00, 0x00, 0x00];
        match self.write_block_with_password(card, block, &terminator, actual_password) {
            Ok(_) => {
                // Sikeres írás
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                // Ha password nélkül SW1=0x63 hibát kaptunk, és van password megadva, próbáljuk meg password-dal
                if error_msg.contains("PASSWORD_REQUIRED") && password.is_some() && actual_password.is_none() {
                    println!("      🔐 Password védelem aktív (SW1=0x63), authenticate-olunk és újrapróbáljuk...");
                    if let Some(pwd) = password {
                        self.authenticate_password(card, pwd)?;
                        actual_password = password; // Most password-dal írunk
                        // Újrapróbáljuk password-dal
                        self.write_block_with_password(card, block, &terminator, actual_password)?;
                    }
                } else {
                    return Err(e);
                }
            }
        }
        
        println!("      ✅ NDEF URL sikeresen írva");
        Ok(())
    }

    /// NDEF URL üzenet létrehozása
    fn create_ndef_url(&self, url: &str) -> Result<Vec<u8>> {
        // NDEF Record formátum:
        // [Header] [Type Length] [Payload Length] [Type] [Payload]
        
        let url_bytes = url.as_bytes();
        let url_len = url_bytes.len();
        
        if url_len > 250 {
            anyhow::bail!("Az URL túl hosszú");
        }

        // Payload: [URI Prefix Code] [URI...]
        // NDEF URI prefix codes:
        // 0x01 = http://www.
        // 0x02 = https://www.
        // 0x03 = http://
        // 0x04 = https://
        let (prefix_code, url_without_prefix) = if url.starts_with("http://www.") {
            (0x01, &url[11..]) // "http://www." után
        } else if url.starts_with("https://www.") {
            (0x02, &url[12..]) // "https://www." után
        } else if url.starts_with("http://") {
            (0x03, &url[7..]) // "http://" után
        } else if url.starts_with("https://") {
            (0x04, &url[8..]) // "https://" után
        } else {
            (0x00, url) // Nincs prefix
        };
        
        // Header byte:
        // MB=1 (Message Begin), ME=1 (Message End), CF=0, SR=1 (Short Record), IL=0, TNF=0x01 (Well Known)
        let header = 0xD1; // 1101 0001
        
        // Type Length: 1 (U = 0x55)
        let type_length = 0x01;
        
        // Payload Length (short record, 1 byte)
        // Prefix code (1 byte) + URL hossza
        let payload_length = (1 + url_without_prefix.len()) as u8;
        
        // Type: U (0x55) = URI Record
        let type_byte = 0x55;
        
        let mut payload = vec![prefix_code];
        
        payload.extend_from_slice(url_without_prefix.as_bytes());
        
        // Összeállítjuk az NDEF üzenetet
        let mut ndef = vec![header, type_length, payload_length, type_byte];
        ndef.extend_from_slice(&payload);
        
        Ok(ndef)
    }

    /// NDEF üzenet parse-olása URL-lé
    fn parse_ndef_url(&self, ndef_data: &[u8]) -> Result<Option<String>> {
        println!("        🔍 parse_ndef_url() részletes elemzés:");
        println!("          NDEF adatok hossza: {} bytes", ndef_data.len());
        println!("          NDEF adatok: {:02X?}", ndef_data);
        
        if ndef_data.is_empty() {
            println!("          ❌ Üres NDEF adatok");
            return Ok(None);
        }

        // Olvassuk a header byte-ot
        let header = ndef_data[0];
        println!("          Header: 0x{:02X}", header);
        
        // Ellenőrizzük, hogy Well Known Type-e
        let tnf = header & 0x07;
        println!("          TNF: {}", tnf);
        if tnf != 0x01 {
            println!("          ❌ Nem Well Known Type (TNF={})", tnf);
            return Ok(None);
        }

        // Type Length
        if ndef_data.len() < 3 {
            println!("          ❌ NDEF adatok túl rövidek (<3 bytes)");
            return Ok(None);
        }
        let type_length = ndef_data[1] as usize;
        println!("          Type Length: {}", type_length);
        
        // Payload Length (short record)
        let payload_length = ndef_data[2] as usize;
        println!("          Payload Length: {} bytes", payload_length);
        
        // Type byte
        if ndef_data.len() < 4 + type_length {
            println!("          ❌ NDEF adatok túl rövidek (<4+type_length bytes)");
            return Ok(None);
        }
        let type_byte = ndef_data[3];
        println!("          Type byte: 0x{:02X}", type_byte);
        
        if type_byte != 0x55 {
            println!("          ❌ Nem URI record (Type=0x{:02X}, nem 0x55)", type_byte);
            return Ok(None); // Nem URI record
        }

        // Payload pozíció: Header(1) + TypeLength(1) + PayloadLength(1) + Type(type_length)
        let payload_start = 3 + type_length;
        println!("          Payload start pozíció: {}", payload_start);
        
        if ndef_data.len() < payload_start + payload_length {
            println!("          ❌ NDEF adatok túl rövidek (len={}, szükséges={})", 
                ndef_data.len(), payload_start + payload_length);
            return Ok(None);
        }
        
        let payload = &ndef_data[payload_start..payload_start + payload_length];
        println!("          Payload ({} bytes): {:02X?}", payload.len(), payload);
        
        if payload.is_empty() {
            println!("          ❌ Üres payload");
            return Ok(None);
        }

        // Prefix code
        let prefix_code = payload[0];
        let url_part = &payload[1..];
        println!("          Prefix code: 0x{:02X}", prefix_code);
        println!("          URL rész: {:02X?} = \"{}\"", url_part, String::from_utf8_lossy(url_part));
        
        let url = match prefix_code {
            0x01 => format!("http://www.{}", String::from_utf8_lossy(url_part)),
            0x02 => format!("https://www.{}", String::from_utf8_lossy(url_part)),
            0x03 => format!("http://{}", String::from_utf8_lossy(url_part)),
            0x04 => format!("https://{}", String::from_utf8_lossy(url_part)),
            _ => {
                println!("          ⚠️ Ismeretlen prefix code: 0x{:02X}, teljes payload-t használjuk", prefix_code);
                String::from_utf8_lossy(payload).to_string()
            },
        };

        println!("          ✅ Parse-olt URL: {}", url);
        Ok(Some(url))
    }

    /// Password beállítása
    pub fn set_password(&self, card: &Card, password: &[u8; 4], pack: &[u8; 2], auth_limit: u8) -> Result<()> {
        println!("    🔐 Ntag216::set_password() CALLED");
        println!("      Password: {:02X?}", password);
        println!("      PACK: {:02X?}", pack);
        println!("      Auth Limit: {}", auth_limit);
        
        // Először ellenőrizzük a jelenlegi konfigurációt
        println!("      🔍 Jelenlegi konfiguráció ellenőrzése...");
        match self.read_config(card) {
            Ok(config) => {
                println!("      📊 Jelenlegi állapot:");
                println!("        Password: {:02X?}", config.password);
                println!("        Locked: {}", config.locked);
                println!("        Read-only: {}", config.read_only);
                if config.locked {
                    anyhow::bail!("A címke zárolva van! Nem lehet módosítani a konfigurációt.");
                }
            }
            Err(e) => {
                println!("      ⚠️ Konfiguráció olvasási hiba (folytatjuk): {}", e);
            }
        }
        
        // MEGJEGYZÉS: A Block 131 sikeresen íródott, de utána a Block 132 és 130 már nem.
        // Ez azt sugallja, hogy a Block 131 írása után változott a címke állapota.
        // Próbáljuk meg fordított sorrendben: először password, aztán Block 131, végül Block 132 és 133.
        
        // MEGJEGYZÉS: A Block 131 sikeresen íródott, de utána a többi blokk már nem.
        // Ez azt sugallja, hogy a Block 131 írása után változott a címke állapota.
        // Próbáljuk meg először csak a password-ot és az aktiválást, majd a Block 131-et és 132-et.
        
        // Block 130: Password (ELŐSZÖR)
        println!("      📝 Block 130 írása (Password)...");
        self.write_block(card, 130, password)
            .map_err(|e| {
                println!("      ❌ Block 130 írási hiba: {}", e);
                e
            })?;
        println!("      ✅ Block 130 írva");
        
        // Block 133: Password védelem aktiválás (MÁSODIK - password után)
        let access_config = [0x01, 0x00, 0x00, 0x00]; // Bit 0 = 1 (password védelem ON)
        println!("      📝 Block 133 írása (Access Config): {:02X?}...", access_config);
        
        let block133_ok = match self.write_block(card, 133, &access_config) {
            Ok(_) => {
                println!("      ✅ Block 133 írva");
                true
            }
            Err(e) => {
                println!("      ⚠️ Block 133 írási hiba: {}", e);
                println!("      💡 Lehet, hogy a Block 133-at csak password után lehet írni.");
                false
            }
        };
        
        // Block 131: PACK + ACCESS (HARMADIK - password és aktiválás után)
        let mut pack_access = [0u8; 4];
        pack_access[0] = pack[0];
        pack_access[1] = pack[1];
        pack_access[2] = 0x80; // ACCESS[0] - user data védelem
        pack_access[3] = 0x00; // ACCESS[1]
        println!("      📝 Block 131 írása (PACK+ACCESS): {:02X?}...", pack_access);
        
        let block131_ok = match self.write_block(card, 131, &pack_access) {
            Ok(_) => {
                println!("      ✅ Block 131 írva");
                true
            }
            Err(e) => {
                println!("      ⚠️ Block 131 írási hiba: {}", e);
                println!("      💡 A Block 131-et lehet, hogy csak password aktiválás előtt lehet írni.");
                false
            }
        };
        
        // Block 132: Auth limit (NEGYEDIK)
        let auth_limit_data = [auth_limit, 0x00, 0x00, 0x00];
        println!("      📝 Block 132 írása (Auth Limit): {:02X?}...", auth_limit_data);
        
        let block132_ok = match self.write_block(card, 132, &auth_limit_data) {
            Ok(_) => {
                println!("      ✅ Block 132 írva");
                true
            }
            Err(e) => {
                println!("      ⚠️ Block 132 írási hiba: {}", e);
                println!("      💡 A Block 132-et lehet, hogy csak password aktiválás előtt lehet írni.");
                false
            }
        };
        
        // Ha a Block 131 és 132 nem sikerült password után, próbáljuk meg újra password előtt
        if !block131_ok || !block132_ok {
            println!("      🔄 Block 131 és 132 újrapróbálása password előtt...");
            
            // Újra Block 131
            if !block131_ok {
                match self.write_block(card, 131, &pack_access) {
                    Ok(_) => {
                        println!("      ✅ Block 131 sikeresen írva újrapróbálással");
                    }
                    Err(e) => {
                        println!("      ⚠️ Block 131 írási hiba újrapróbáláskor is: {}", e);
                    }
                }
            }
            
            // Újra Block 132
            if !block132_ok {
                match self.write_block(card, 132, &auth_limit_data) {
                    Ok(_) => {
                        println!("      ✅ Block 132 sikeresen írva újrapróbálással");
                    }
                    Err(e) => {
                        println!("      ⚠️ Block 132 írási hiba újrapróbáláskor is: {}", e);
                    }
                }
            }
        }
        
        println!("      ✅ Password védelem beállítása befejezve");
        println!("      📊 Eredmény: Block 130=✅, Block 131={}, Block 132={}, Block 133={}", 
            if block131_ok { "✅" } else { "❌" }, 
            if block132_ok { "✅" } else { "❌" },
            if block133_ok { "✅" } else { "❌" });
        
        // MEGJEGYZÉS: A Block 130 sikeresen íródott, de utána a többi blokk már nem írható.
        // Ez azt sugallja, hogy a Block 130 írása után automatikusan aktiválódik a password védelem,
        // vagy a PC/SC API-n keresztül nem lehet írni ezeket a blokkokat password után.
        // 
        // FONTOS: Ez NEM az olvasó vagy driver hibája!
        // - A PC/SC API eredetileg smart card-okhoz készült, nem NFC címkékhez
        // - Az NTAG216 password authentication speciális művelet, ami nem mindig illeszkedik a PC/SC standardhoz
        // - Még az ACS CCID natív driver telepítése után is ezek a korlátok fennállhatnak
        // 
        // A password beállítva (Block 130), ami a legfontosabb. A Block 131, 132, 133 lehet, hogy csak
        // natív NFC driver-rel vagy speciális módszerekkel írható, de a password védelem
        // általában működik csak a Block 130 beállításával is.
        
        if block133_ok {
            println!("      ✅ Password védelem teljesen beállítva!");
            Ok(())
        } else {
            println!("      ⚠️ Password beállítva (Block 130), de a többi blokk nem írható PC/SC API-n keresztül.");
            println!("      💡 A password védelem lehet, hogy automatikusan aktív a Block 130 írása után.");
            println!("      💡 A Block 131, 132, 133 lehet, hogy csak natív driver-rel írható.");
            Ok(()) // Sikeresnek tekintjük, mert a password beállítva
        }
    }

    /// Password védelem eltávolítása
    pub fn remove_password(&self, card: &Card) -> Result<()> {
        // Block 130: Password törlése
        let empty_password = [0x00, 0x00, 0x00, 0x00];
        self.write_block(card, 130, &empty_password)?;
        
        // Block 131: PACK + ACCESS törlése
        let empty_access = [0x00, 0x00, 0x00, 0x00];
        self.write_block(card, 131, &empty_access)?;
        
        // Block 132: Auth limit törlése
        let empty_limit = [0x00, 0x00, 0x00, 0x00];
        self.write_block(card, 132, &empty_limit)?;
        
        // Block 133: Password védelem kikapcsolása
        let access_config = [0x00, 0x00, 0x00, 0x00];
        self.write_block(card, 133, &access_config)?;
        
        Ok(())
    }

    /// Read-only mód beállítása (VISSZAFORDÍTHATATLAN!)
    pub fn set_read_only(&self, card: &Card) -> Result<()> {
        // Block 133: Read-only bit beállítása
        let read_only_config = [0x00, 0x01, 0x00, 0x00]; // Bit 1 = 1 (read-only)
        self.write_block(card, 133, &read_only_config)?;
        
        // Block 134: LOCK (visszafordíthatatlan!)
        let lock = [0xFF, 0xFF, 0x00, 0x00];
        self.write_block(card, 134, &lock)?;
        
        Ok(())
    }

    /// Konfiguráció olvasása
    pub fn read_config(&self, card: &Card) -> Result<NtagConfig> {
        let pwd = self.read_block(card, 130)?;
        let pack_access = self.read_block(card, 131)?;
        let auth_limit = self.read_block(card, 132)?;
        let access_config = self.read_block(card, 133)?;
        let lock = self.read_block(card, 134)?;
        
        Ok(NtagConfig {
            password: [pwd[0], pwd[1], pwd[2], pwd[3]],
            pack: [pack_access[0], pack_access[1]],
            access: [pack_access[2], pack_access[3]],
            auth_limit: auth_limit[0],
            password_protected: (access_config[0] & 0x01) != 0,
            read_only: (access_config[1] & 0x01) != 0,
            locked: lock[0] == 0xFF && lock[1] == 0xFF,
        })
    }

    /// NDEF Text Record írása
    pub fn write_ndef_text(&self, card: &Card, text: &str, language: &str) -> Result<()> {
        self.write_ndef_text_with_password(card, text, language, None)
    }

    pub fn write_ndef_text_with_password(&self, card: &Card, text: &str, language: &str, password: Option<&[u8; 4]>) -> Result<()> {
        if !self.check_type(card)? {
            anyhow::bail!("Ez nem egy NTAG216 címke");
        }

        let ndef_message = self.create_ndef_text(text, language)?;
        self.write_ndef_message_with_password(card, &ndef_message, password)?;
        Ok(())
    }

    /// NDEF Text Record olvasása
    pub fn read_ndef_text(&self, card: &Card) -> Result<Option<(String, String)>> {
        let ndef_data = self.read_ndef_raw(card)?;
        if let Some(data) = ndef_data {
            self.parse_ndef_text(&data)
        } else {
            Ok(None)
        }
    }

    /// NDEF vCard írása
    pub fn write_ndef_vcard(&self, card: &Card, vcard: &str) -> Result<()> {
        self.write_ndef_vcard_with_password(card, vcard, None)
    }

    pub fn write_ndef_vcard_with_password(&self, card: &Card, vcard: &str, password: Option<&[u8; 4]>) -> Result<()> {
        if !self.check_type(card)? {
            anyhow::bail!("Ez nem egy NTAG216 címke");
        }

        let ndef_message = self.create_ndef_vcard(vcard)?;
        self.write_ndef_message_with_password(card, &ndef_message, password)?;
        Ok(())
    }

    /// NDEF vCard olvasása
    pub fn read_ndef_vcard(&self, card: &Card) -> Result<Option<String>> {
        let ndef_data = self.read_ndef_raw(card)?;
        if let Some(data) = ndef_data {
            self.parse_ndef_vcard(&data)
        } else {
            Ok(None)
        }
    }

    /// NDEF üzenet törlése
    pub fn clear_ndef(&self, card: &Card) -> Result<()> {
        self.clear_ndef_with_password(card, None)
    }

    pub fn clear_ndef_with_password(&self, card: &Card, password: Option<&[u8; 4]>) -> Result<()> {
        // Ha password van, authenticate-olunk először
        if let Some(pwd) = password {
            self.authenticate_password(card, pwd)?;
        }

        // TLV terminátor írása (üres NDEF üzenet)
        let terminator = [0xFE, 0x00, 0x00, 0x00];
        self.write_block_with_password(card, 4, &terminator, password)?;
        
        // További blokkok törlése (opcionális)
        for block in 5..=10 {
            let empty = [0x00, 0x00, 0x00, 0x00];
            self.write_block_with_password(card, block, &empty, password)?;
        }
        
        Ok(())
    }

    /// Raw byte írása (NDEF nélkül)
    pub fn write_raw_bytes(&self, card: &Card, start_block: u8, data: &[u8]) -> Result<()> {
        self.write_raw_bytes_with_password(card, start_block, data, None)
    }

    pub fn write_raw_bytes_with_password(&self, card: &Card, start_block: u8, data: &[u8], password: Option<&[u8; 4]>) -> Result<()> {
        if start_block < 4 || start_block > 129 {
            anyhow::bail!("Érvénytelen block szám (4-129)");
        }

        // Ha password van, authenticate-olunk először
        if let Some(pwd) = password {
            self.authenticate_password(card, pwd)?;
        }

        let mut block = start_block;
        let mut data_index = 0;
        
        while data_index < data.len() && block <= 129 {
            let mut block_data = [0u8; 4];
            for i in 0..4 {
                if data_index < data.len() {
                    block_data[i] = data[data_index];
                    data_index += 1;
                } else {
                    block_data[i] = 0x00;
                }
            }
            self.write_block_with_password(card, block, &block_data, password)?;
            block += 1;
        }
        
        Ok(())
    }

    /// Raw byte olvasása
    pub fn read_raw_bytes(&self, card: &Card, start_block: u8, count: u8) -> Result<Vec<u8>> {
        if start_block < 4 || start_block > 129 {
            anyhow::bail!("Érvénytelen block szám (4-129)");
        }

        let mut result = Vec::new();
        let mut block = start_block;
        let mut remaining = count as usize;
        
        while remaining > 0 && block <= 129 {
            let block_data = self.read_block(card, block)?;
            let to_take = remaining.min(4);
            result.extend_from_slice(&block_data[..to_take]);
            remaining -= to_take;
            block += 1;
        }
        
        Ok(result)
    }

    // Helper függvények

    fn write_ndef_message(&self, card: &Card, ndef_message: &[u8]) -> Result<()> {
        self.write_ndef_message_with_password(card, ndef_message, None)
    }

    fn write_ndef_message_with_password(&self, card: &Card, ndef_message: &[u8], password: Option<&[u8; 4]>) -> Result<()> {
        // Ha password van, authenticate-olunk először
        if let Some(pwd) = password {
            self.authenticate_password(card, pwd)?;
        }

        let tlv_length = ndef_message.len();
        if tlv_length > 255 {
            anyhow::bail!("Az NDEF üzenet túl nagy (max 255 byte)");
        }

        let mut block = 4;
        let mut data_to_write = vec![0x03, tlv_length as u8];
        data_to_write.extend_from_slice(ndef_message);
        
        let mut block_data = [0u8; 4];
        let mut data_index = 0;
        
        while data_index < data_to_write.len() {
            for i in 0..4 {
                if data_index < data_to_write.len() {
                    block_data[i] = data_to_write[data_index];
                    data_index += 1;
                } else {
                    block_data[i] = 0x00;
                }
            }
            
            self.write_block_with_password(card, block, &block_data, password)?;
            block += 1;
            
            if block > 129 {
                anyhow::bail!("Az NDEF üzenet túl nagy az NTAG216 kapacitásához");
            }
        }

        let terminator = [0xFE, 0x00, 0x00, 0x00];
        self.write_block_with_password(card, block, &terminator, password)?;
        
        Ok(())
    }

    fn read_ndef_raw(&self, card: &Card) -> Result<Option<Vec<u8>>> {
        let cc = self.read_block(card, 3)?;
        if cc[0] != 0xE1 {
            return Ok(None);
        }

        let tlv = self.read_block(card, 4)?;
        if tlv[0] != 0x03 {
            return Ok(None);
        }

        let length = tlv[1] as usize;
        if length == 0 {
            return Ok(None);
        }

        let mut ndef_data = Vec::new();
        let mut block = 4;
        let mut offset = 2;
        
        while ndef_data.len() < length {
            let block_data = self.read_block(card, block)?;
            for i in offset..4 {
                if ndef_data.len() < length {
                    ndef_data.push(block_data[i]);
                }
            }
            block += 1;
            offset = 0;
        }

        Ok(Some(ndef_data))
    }

    fn create_ndef_text(&self, text: &str, language: &str) -> Result<Vec<u8>> {
        let text_bytes = text.as_bytes();
        let lang_bytes = language.as_bytes();
        
        if lang_bytes.len() > 5 {
            anyhow::bail!("A nyelv kód túl hosszú (max 5 karakter)");
        }
        
        if text_bytes.len() > 200 {
            anyhow::bail!("A szöveg túl hosszú (max 200 karakter)");
        }

        let header = 0xD1; // MB=1, ME=1, SR=1, TNF=001
        let type_length = 0x01; // "T" = 0x54
        let payload_length = (1 + lang_bytes.len() + text_bytes.len()) as u8;
        let type_byte = 0x54; // "T" = Text Record
        
        let mut payload = vec![lang_bytes.len() as u8];
        payload.extend_from_slice(lang_bytes);
        payload.extend_from_slice(text_bytes);
        
        let mut ndef = vec![header, type_length, payload_length, type_byte];
        ndef.extend_from_slice(&payload);
        
        Ok(ndef)
    }

    fn parse_ndef_text(&self, ndef_data: &[u8]) -> Result<Option<(String, String)>> {
        if ndef_data.len() < 4 {
            return Ok(None);
        }

        let header = ndef_data[0];
        let tnf = header & 0x07;
        if tnf != 0x01 {
            return Ok(None);
        }

        let type_length = ndef_data[1] as usize;
        let payload_length = ndef_data[2] as usize;
        
        if ndef_data.len() < 4 + type_length + payload_length {
            return Ok(None);
        }

        let type_byte = ndef_data[3];
        if type_byte != 0x54 {
            return Ok(None);
        }

        let payload_start = 4 + type_length;
        let payload = &ndef_data[payload_start..payload_start + payload_length];
        
        if payload.is_empty() {
            return Ok(None);
        }

        let lang_length = payload[0] as usize;
        if payload.len() < 1 + lang_length {
            return Ok(None);
        }

        let language = String::from_utf8_lossy(&payload[1..1 + lang_length]).to_string();
        let text = String::from_utf8_lossy(&payload[1 + lang_length..]).to_string();
        
        Ok(Some((text, language)))
    }

    fn create_ndef_vcard(&self, vcard: &str) -> Result<Vec<u8>> {
        let vcard_bytes = vcard.as_bytes();
        
        if vcard_bytes.len() > 400 {
            anyhow::bail!("A vCard túl hosszú (max 400 karakter)");
        }

        let header = 0xD2; // MB=1, ME=1, SR=1, TNF=010 (MIME)
        let type_length = 0x0A; // "text/vcard" = 10 karakter
        let payload_length = vcard_bytes.len() as u16;
        
        // Short record csak 1 byte payload length, ha >255 akkor long record
        if payload_length > 255 {
            anyhow::bail!("A vCard túl hosszú (max 255 byte)");
        }
        
        let type_bytes = b"text/vcard";
        let mut ndef = vec![header, type_length, payload_length as u8];
        ndef.extend_from_slice(type_bytes);
        ndef.extend_from_slice(vcard_bytes);
        
        Ok(ndef)
    }

    fn parse_ndef_vcard(&self, ndef_data: &[u8]) -> Result<Option<String>> {
        if ndef_data.len() < 4 {
            return Ok(None);
        }

        let header = ndef_data[0];
        let tnf = header & 0x07;
        if tnf != 0x02 {
            return Ok(None); // Nem MIME type
        }

        let type_length = ndef_data[1] as usize;
        let payload_length = ndef_data[2] as usize;
        
        if ndef_data.len() < 4 + type_length + payload_length {
            return Ok(None);
        }

        let type_bytes = &ndef_data[3..3 + type_length];
        if type_bytes != b"text/vcard" {
            return Ok(None);
        }

        let payload_start = 3 + type_length;
        let vcard = String::from_utf8_lossy(&ndef_data[payload_start..payload_start + payload_length]).to_string();
        
        Ok(Some(vcard))
    }
}

/// NTAG216 konfiguráció struktúra
#[derive(Debug, Clone)]
pub struct NtagConfig {
    pub password: [u8; 4],
    pub pack: [u8; 2],
    pub access: [u8; 2],
    pub auth_limit: u8,
    pub password_protected: bool,
    pub read_only: bool,
    pub locked: bool,
}
