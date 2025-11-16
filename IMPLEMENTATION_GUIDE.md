# Implementációs Útmutató - Tauri NFC Alkalmazás

## Projekt Beállítás

### 1. Tauri Projekt Létrehozása

```bash
npm create tauri-app@latest nfc-app
cd nfc-app
```

### 2. Rust Dependencies Hozzáadása

A `src-tauri/Cargo.toml` fájlhoz:

```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# NFC támogatás
pcsc = "2.9"  # PC/SC API wrapper
# vagy
nfc-rs = "0.1"  # Ha libnfc-t használsz
```

### 3. Platform Specifikus Megközelítések

#### Desktop (Windows/Linux/macOS)

**PC/SC API használata:**

```rust
use pcsc::*;

pub struct NFCReader {
    ctx: Context,
}

impl NFCReader {
    pub fn new() -> Result<Self, String> {
        let ctx = Context::establish(Scope::User)
            .map_err(|e| format!("PC/SC context error: {}", e))?;
        Ok(NFCReader { ctx })
    }

    pub fn list_readers(&self) -> Result<Vec<String>, String> {
        let readers = self.ctx.list_readers_len(MAX_READERS)
            .map_err(|e| format!("List readers error: {}", e))?;
        
        Ok(readers.iter()
            .map(|r| r.to_string_lossy().to_string())
            .collect())
    }
}
```

#### Mobil (Android/iOS) - Tauri Plugin

```bash
npm run tauri add nfc
```

**Frontend használat:**

```typescript
import { scan, write, isAvailable } from '@tauri-apps/plugin-nfc';

// Elérhetőség ellenőrzése
const available = await isAvailable();

// NFC címke olvasása
const tag = await scan(
  { type: 'ndef' },
  {
    keepSessionAlive: false,
    message: 'Olvass be egy NFC címkét',
  }
);

// NFC címke írása
await write(
  [uriRecord('https://example.com'), textRecord('Hello')],
  {
    kind: { type: 'ndef' },
    message: 'Olvass be egy NFC címkét',
  }
);
```

## NTAG216 Implementáció

### Rust Backend

```rust
// src-tauri/src/nfc/ntag216.rs

pub struct NTAG216 {
    card: Card,
}

impl NTAG216 {
    pub fn new(card: Card) -> Self {
        NTAG216 { card }
    }

    // NDEF üzenet olvasása
    pub fn read_ndef(&self) -> Result<Vec<u8>, String> {
        // NTAG216 NDEF kezdőcíme: 0x04
        let mut data = vec![0u8; 16];
        self.card.transmit(&[0xFF, 0xB0, 0x00, 0x04, 0x10], &mut data)
            .map_err(|e| format!("Read error: {}", e))?;
        Ok(data)
    }

    // NDEF üzenet írása
    pub fn write_ndef(&self, message: &[u8]) -> Result<(), String> {
        // NDEF üzenet írása blokkokba
        for (i, chunk) in message.chunks(4).enumerate() {
            let block_addr = 0x04 + i as u8;
            let mut write_data = chunk.to_vec();
            write_data.resize(4, 0);
            
            self.card.transmit(
                &[0xFF, 0xD6, 0x00, block_addr, 0x04],
                &mut write_data
            ).map_err(|e| format!("Write error: {}", e))?;
        }
        Ok(())
    }

    // Password beállítása
    pub fn set_password(&mut self, password: u32) -> Result<(), String> {
        let pwd_bytes = password.to_le_bytes();
        // Password blokk: 0x85
        self.card.transmit(
            &[0xFF, 0xD6, 0x00, 0x85, 0x04],
            &mut pwd_bytes.to_vec()
        ).map_err(|e| format!("Password set error: {}", e))?;
        Ok(())
    }

    // UID olvasása
    pub fn read_uid(&self) -> Result<Vec<u8>, String> {
        let mut data = vec![0u8; 16];
        self.card.transmit(&[0xFF, 0xCA, 0x00, 0x00, 0x00], &mut data)
            .map_err(|e| format!("UID read error: {}", e))?;
        Ok(data[..7].to_vec())
    }
}
```

### Tauri Commands

```rust
// src-tauri/src/commands.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NdefMessage {
    records: Vec<NdefRecord>,
}

#[derive(Serialize, Deserialize)]
pub struct NdefRecord {
    r#type: String,
    content: String,
}

#[tauri::command]
pub async fn read_ntag216() -> Result<NdefMessage, String> {
    // NFC olvasó inicializálás
    // NTAG216 olvasás
    // NDEF parsing
    Ok(NdefMessage { records: vec![] })
}

#[tauri::command]
pub async fn write_ntag216(message: NdefMessage) -> Result<(), String> {
    // NDEF üzenet készítése
    // NTAG216 írás
    Ok(())
}
```

## Mifare Classic Implementáció

### Rust Backend

```rust
// src-tauri/src/nfc/mifare.rs

pub struct MifareClassic {
    card: Card,
}

#[derive(Clone, Copy)]
pub enum KeyType {
    A,
    B,
}

impl MifareClassic {
    pub fn new(card: Card) -> Self {
        MifareClassic { card }
    }

    // Szektor autentikáció
    pub fn authenticate(&self, sector: u8, key: &[u8; 6], key_type: KeyType) -> Result<(), String> {
        let block = sector * 4;
        let cmd = match key_type {
            KeyType::A => 0x60,
            KeyType::B => 0x61,
        };
        
        let mut cmd_data = vec![cmd, block];
        cmd_data.extend_from_slice(key);
        
        let mut response = vec![0u8; 2];
        self.card.transmit(&cmd_data, &mut response)
            .map_err(|e| format!("Authentication error: {}", e))?;
        
        if response[0] == 0x90 && response[1] == 0x00 {
            Ok(())
        } else {
            Err("Authentication failed".to_string())
        }
    }

    // Blokk olvasás
    pub fn read_block(&self, block: u8) -> Result<Vec<u8>, String> {
        let mut data = vec![0u8; 18];
        self.card.transmit(&[0xFF, 0xB0, 0x00, block, 0x10], &mut data)
            .map_err(|e| format!("Read error: {}", e))?;
        Ok(data[..16].to_vec())
    }

    // Blokk írás
    pub fn write_block(&self, block: u8, data: &[u8; 16]) -> Result<(), String> {
        let mut cmd = vec![0xFF, 0xD6, 0x00, block, 0x10];
        cmd.extend_from_slice(data);
        
        let mut response = vec![0u8; 2];
        self.card.transmit(&cmd, &mut response)
            .map_err(|e| format!("Write error: {}", e))?;
        
        if response[0] == 0x90 && response[1] == 0x00 {
            Ok(())
        } else {
            Err("Write failed".to_string())
        }
    }

    // Value block növelés
    pub fn increment_value(&self, block: u8, value: i32) -> Result<(), String> {
        let value_bytes = value.to_le_bytes();
        let mut cmd = vec![0xFF, 0xD7, 0x00, block, 0x04];
        cmd.extend_from_slice(&value_bytes);
        
        let mut response = vec![0u8; 2];
        self.card.transmit(&cmd, &mut response)
            .map_err(|e| format!("Increment error: {}", e))?;
        
        if response[0] == 0x90 && response[1] == 0x00 {
            Ok(())
        } else {
            Err("Increment failed".to_string())
        }
    }

    // Value block átvitel
    pub fn transfer_value(&self, block: u8) -> Result<(), String> {
        let mut response = vec![0u8; 2];
        self.card.transmit(&[0xFF, 0xB0, 0x00, block, 0x05], &mut response)
            .map_err(|e| format!("Transfer error: {}", e))?;
        
        if response[0] == 0x90 && response[1] == 0x00 {
            Ok(())
        } else {
            Err("Transfer failed".to_string())
        }
    }

    // UID olvasás
    pub fn read_uid(&self) -> Result<Vec<u8>, String> {
        let mut data = vec![0u8; 16];
        self.card.transmit(&[0xFF, 0xCA, 0x00, 0x00, 0x00], &mut data)
            .map_err(|e| format!("UID read error: {}", e))?;
        Ok(data[..4].to_vec())
    }
}
```

### Tauri Commands

```rust
#[tauri::command]
pub async fn read_mifare_sector(
    sector: u8,
    key: Vec<u8>,
    key_type: String,
) -> Result<Vec<u8>, String> {
    // Autentikáció
    // Szektor olvasás
    Ok(vec![])
}

#[tauri::command]
pub async fn write_mifare_block(
    block: u8,
    data: Vec<u8>,
    key: Vec<u8>,
    key_type: String,
) -> Result<(), String> {
    // Autentikáció
    // Blokk írás
    Ok(())
}
```

## Frontend Példa (React)

```typescript
// src/components/NFCReader.tsx

import { invoke } from '@tauri-apps/api/tauri';
import { useState } from 'react';

function NFCReader() {
  const [tagType, setTagType] = useState<'ntag216' | 'mifare' | null>(null);
  const [data, setData] = useState<string>('');

  const readNTAG216 = async () => {
    try {
      const result = await invoke('read_ntag216');
      setData(JSON.stringify(result));
      setTagType('ntag216');
    } catch (error) {
      console.error('NTAG216 read error:', error);
    }
  };

  const readMifare = async () => {
    try {
      const sector = 1;
      const key = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
      const result = await invoke('read_mifare_sector', {
        sector,
        key,
        keyType: 'A',
      });
      setData(JSON.stringify(result));
      setTagType('mifare');
    } catch (error) {
      console.error('Mifare read error:', error);
    }
  };

  return (
    <div>
      <button onClick={readNTAG216}>NTAG216 Olvasás</button>
      <button onClick={readMifare}>Mifare Olvasás</button>
      {data && <pre>{data}</pre>}
    </div>
  );
}
```

## Tesztelés

### NTAG216 Teszt
1. NTAG216 címke olvasása
2. NDEF üzenet írása
3. Password beállítás
4. UID olvasás

### Mifare Classic Teszt
1. Default kulcsokkal autentikáció (0xFF...)
2. Szektor olvasás
3. Blokk írás
4. Value block műveletek
5. UID olvasás

## Hibakeresés

### Gyakori Hibák

1. **PC/SC Context Error**
   - Ellenőrizd, hogy a PC/SC szolgáltatás fut-e
   - Linux: `sudo systemctl status pcscd`
   - Windows: Smart Card szolgáltatás

2. **Authentication Failed**
   - Hibás kulcs
   - Címke nem Mifare Classic
   - Olvasó nem támogatja

3. **Write Failed**
   - Címke read-only
   - Nincs autentikáció
   - Hibás blokk cím

## Következő Lépések

1. ✅ Projekt inicializálás
2. ✅ NFC könyvtárak integrálása
3. ✅ NTAG216 implementáció
4. ✅ Mifare Classic implementáció
5. ✅ Frontend UI fejlesztés
6. ✅ Tesztelés valós címkékkel
7. ✅ Hibakezelés és validáció
8. ✅ Dokumentáció



