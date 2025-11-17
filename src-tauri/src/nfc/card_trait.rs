use anyhow::Result;

/// Trait az NFC kártya kommunikációhoz
/// Ez lehetővé teszi, hogy az NTAG216 implementáció működjön PC/SC és libnfc egyaránt
pub trait NfcCard {
    /// APDU parancs küldése a címkének
    fn transmit(&self, apdu: &[u8]) -> Result<Vec<u8>>;
}

// PC/SC Card implementáció
#[cfg(feature = "pcsc")]
impl NfcCard for pcsc::Card {
    fn transmit(&self, apdu: &[u8]) -> Result<Vec<u8>> {
        use anyhow::Context;
        let mut response_buffer = [0u8; 256];
        let response = self.transmit(apdu, &mut response_buffer)
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
}

// libnfc wrapper implementáció
#[cfg(feature = "libnfc")]
pub struct LibnfcCardWrapper {
    reader: std::sync::Arc<std::sync::Mutex<crate::nfc::reader_libnfc::LibnfcReader>>,
}

#[cfg(feature = "libnfc")]
impl LibnfcCardWrapper {
    pub fn new(reader: std::sync::Arc<std::sync::Mutex<crate::nfc::reader_libnfc::LibnfcReader>>) -> Self {
        Self { reader }
    }
}

#[cfg(feature = "libnfc")]
impl NfcCard for LibnfcCardWrapper {
    fn transmit(&self, apdu: &[u8]) -> Result<Vec<u8>> {
        use anyhow::Context;
        let mut reader = self.reader.lock().unwrap();
        let response = reader.transmit(apdu)
            .context("Nem sikerült kommunikálni az NFC címkével libnfc-n keresztül")?;
        
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
}




