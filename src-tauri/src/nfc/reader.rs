use anyhow::{Context, Result};
use pcsc::{Card, Context as PcscContext, Protocols, Scope, ShareMode};

pub struct NfcReader {
    ctx: PcscContext,
}

impl NfcReader {
    pub fn new() -> Result<Self> {
        let ctx = PcscContext::establish(Scope::User)
            .context("Nem sikerült csatlakozni a PC/SC szolgáltatáshoz")?;
        
        Ok(NfcReader { ctx })
    }

    pub fn connect(&self) -> Result<Card> {
        println!("  📡 NfcReader::connect() CALLED");
        let mut buffer = [0u8; 2048];
        println!("  📋 Olvasók listázása...");
        let mut readers = self.ctx.list_readers(&mut buffer)
            .context("Nem sikerült listázni az olvasókat")?;
        
        println!("  🔍 Olvasók keresése...");
        // ReaderNames már egy iterator-like struktúra
        let reader = readers
            .next()
            .ok_or_else(|| {
                println!("  ❌ Nincs NFC olvasó csatlakoztatva");
                anyhow::anyhow!("Nincs NFC olvasó csatlakoztatva")
            })?;
        
        println!("  ✅ Olvasó találva: {:?}", reader);
        println!("  🔌 Címke csatlakoztatása...");
        let card = self.ctx.connect(&reader, ShareMode::Shared, Protocols::ANY)
            .context("Nem sikerült csatlakozni az NFC címkéhez. Kérlek helyezd a címkét az olvasóra.")?;
        
        println!("  ✅ Címke csatlakoztatva");
        Ok(card)
    }

    pub fn list_readers(&self) -> Result<Vec<String>> {
        let mut buffer = [0u8; 2048];
        let readers = self.ctx.list_readers(&mut buffer)
            .context("Nem sikerült listázni az olvasókat")?;
        
        // ReaderNames már egy iterator-like struktúra, CStr-t String-gé konvertáljuk
        let mut result = Vec::new();
        for reader in readers {
            result.push(reader.to_string_lossy().to_string());
        }
        Ok(result)
    }
}

impl Default for NfcReader {
    fn default() -> Self {
        Self::new().expect("Nem sikerült inicializálni az NFC olvasót")
    }
}

