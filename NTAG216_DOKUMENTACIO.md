# NTAG216 Részletes Dokumentáció

## 📋 Tartalomjegyzék

1. [NTAG216 Áttekintés](#ntag216-áttekintés)
2. [Hogyan Működik](#hogyan-működik)
3. [Technikai Specifikáció](#technikai-specifikáció)
4. [Memória Struktúra](#memória-struktúra)
5. [NDEF Formátum](#ndef-formátum)
6. [APDU Parancsok](#apdu-parancsok)
7. [Mit Lehet Megtenni](#mit-lehet-megtenni)
8. [Implementáció Részletei](#implementáció-részletei)

---

## NTAG216 Áttekintés

Az **NTAG216** egy NFC Type 2 Tag (ISO14443A kompatibilis), amelyet az NXP (Nexperia) gyárt. Ez egy passzív NFC címke, ami azt jelenti, hogy nincs saját áramforrása - az NFC olvasó elektromágneses mezőjéből táplálkozik.

### Főbb Jellemzők

- ✅ **Kapacitás:** 888 bytes felhasználói adat
- ✅ **Blokkok:** 135 blocks (minden block 4 bytes)
- ✅ **Frekvencia:** 13.56 MHz
- ✅ **Olvasási távolság:** ~10 cm (olvasótól függően)
- ✅ **Írási távolság:** ~5-7 cm
- ✅ **NDEF támogatás:** Igen, natív NDEF formátum
- ✅ **Védelem:** Password védelem, read-only mód

---

## Hogyan Működik

### 1. Fizikai Működés

```
┌─────────────┐         ┌──────────────┐
│ NFC Olvasó  │ ◄────►  │  NTAG216     │
│             │  RF     │  Címke       │
│ (13.56 MHz) │         │  (Passzív)   │
└─────────────┘         └──────────────┘
```

1. **Energiaátvitel:** Az NFC olvasó 13.56 MHz-es rádiófrekvenciás jelet küld
2. **Indukció:** Az NTAG216 antennája ezt az energiát használja működéshez
3. **Kommunikáció:** A címke válaszol az olvasó parancsaira
4. **Adatátvitel:** Az adatok modulált RF jeleken keresztül mennek

### 2. Kommunikációs Protokoll

Az alkalmazás **PC/SC (Personal Computer/Smart Card)** API-t használ:

```
Alkalmazás → PC/SC Driver → USB NFC Olvasó → NTAG216 Címke
```

**Lépések:**
1. PC/SC kontextus létrehozása
2. NFC olvasó detektálása
3. Címke csatlakoztatása (amikor az olvasóra helyezed)
4. APDU parancsok küldése
5. Válaszok fogadása

---

## Technikai Specifikáció

### Memória Felépítése

```
┌─────────────────────────────────────────┐
│ Block 0-3:   UID és Manufacturer Data   │ (16 bytes - READ ONLY)
├─────────────────────────────────────────┤
│ Block 4-129: User Data                 │ (504 bytes - ÍRHATÓ)
├─────────────────────────────────────────┤
│ Block 130-134: Configuration Pages     │ (20 bytes - Konfiguráció)
└─────────────────────────────────────────┘

ÖSSZESEN: 135 blocks × 4 bytes = 540 bytes
FELHASZNÁLHATÓ: 126 blocks × 4 bytes = 504 bytes (NDEF-hez)
```

### Block Részletek

**Block 0-2: UID (Unique Identifier)**
- 7 bytes UID
- READ ONLY - nem módosítható

**Block 3: Capability Container (CC)**
- `[E1 10 12 00]` - NTAG216 azonosító
- `E1` = NDEF magic number
- `10` = NDEF Version 1.0
- `12` = Tag size indicator
- `00` = Reserved

**Block 4-129: User Data**
- Itt tárolódik az NDEF üzenet
- TLV (Tag-Length-Value) formátumban
- Maximum 504 bytes

**Block 130-134: Configuration Pages** (20 bytes összesen)
- Password beállítások
- Access control
- Read-only beállítások

**Részletes felépítés:**

```
Block 130: Password (PWD)
┌─────────────────────────────────────────┐
│ Byte 0: PWD[0] (Password byte 0)        │
│ Byte 1: PWD[1] (Password byte 1)        │
│ Byte 2: PWD[2] (Password byte 2)        │
│ Byte 3: PWD[3] (Password byte 3)       │
└─────────────────────────────────────────┘
Alapértelmezett: [00 00 00 00] (nincs password)

Block 131: Password Acknowledge (PACK) + Access
┌─────────────────────────────────────────┐
│ Byte 0: PACK[0] (Password ACK byte 0)   │
│ Byte 1: PACK[1] (Password ACK byte 1)   │
│ Byte 2: ACCESS[0] (Access byte 0)      │
│ Byte 3: ACCESS[1] (Access byte 1)      │
└─────────────────────────────────────────┘
Alapértelmezett: [00 00 00 00]

Block 132: Authentication Limit
┌─────────────────────────────────────────┐
│ Byte 0: AUTH_LIMIT (max próbálkozások) │
│ Byte 1: Reserved (0x00)                 │
│ Byte 2: Reserved (0x00)                │
│ Byte 3: Reserved (0x00)                │
└─────────────────────────────────────────┘
Alapértelmezett: [00 00 00 00]

Block 133: Access Configuration
┌─────────────────────────────────────────┐
│ Byte 0: NFC_CNT_PWD_PROT (bit 0)       │
│         NFC_CNT_PWD_PROT = 1 → Password │
│         védett írás                     │
│ Byte 1: NFC_CNT_READ_CNT (bit 0)       │
│         NFC_CNT_READ_CNT = 1 → Read-only │
│         NFC counter                     │
│ Byte 2: Reserved (0x00)                │
│ Byte 3: Reserved (0x00)                │
└─────────────────────────────────────────┘
Alapértelmezett: [00 00 00 00]

Block 134: Configuration Lock
┌─────────────────────────────────────────┐
│ Byte 0: CFG_LOCK[0] (Lock byte 0)      │
│ Byte 1: CFG_LOCK[1] (Lock byte 1)      │
│ Byte 2: Reserved (0x00)                │
│ Byte 3: Reserved (0x00)                │
└─────────────────────────────────────────┘
Alapértelmezett: [00 00 00 00]
⚠️ Ha beállítod, VISSZAFORDÍTHATATLAN!
```

**Példa Konfigurációk:**

**1. Password védelem beállítása:**
```
Block 130: [12 34 56 78]  ← Password: 0x12345678
Block 131: [80 80 00 00]  ← PACK + ACCESS (alapértelmezett)
Block 132: [03 00 00 00]  ← Max 3 próbálkozás
Block 133: [01 00 00 00]  ← Password védelem aktív
Block 134: [00 00 00 00]  ← Nincs lock (még módosítható)
```

**2. Read-only mód (visszafordíthatatlan!):**
```
Block 130: [00 00 00 00]  ← Nincs password
Block 131: [00 00 00 00]  ← Alapértelmezett
Block 132: [00 00 00 00]  ← Alapértelmezett
Block 133: [00 01 00 00]  ← NFC counter read-only
Block 134: [FF FF 00 00]  ← ⚠️ LOCK! Visszafordíthatatlan!
```

**3. Password védett írás:**
```
Block 130: [AB CD EF 12]  ← Password
Block 131: [80 80 00 00]  ← PACK + ACCESS
Block 132: [05 00 00 00]  ← Max 5 próbálkozás
Block 133: [01 00 00 00]  ← Password védelem ON
Block 134: [00 00 00 00]  ← Nincs lock
```

📚 **Részletes vizuális útmutató:** [`CONFIGURATION_BLOKKOK.md`](CONFIGURATION_BLOKKOK.md)
- Vizuális ábrák
- Részletes magyarázatok byte szinten
- Gyakorlati példák
- Kód példák

---

## NDEF Formátum

### Mi az NDEF?

**NDEF** = NFC Data Exchange Format

Ez egy standard formátum, amit az NFC eszközök használnak az adatok tárolására. Az NTAG216 natív NDEF támogatással rendelkezik.

### NDEF Record Struktúra

```
┌─────────────────────────────────────────┐
│ Header Byte (1 byte)                    │
│ ├─ MB (Message Begin)                   │
│ ├─ ME (Message End)                     │
│ ├─ CF (Chunk Flag)                      │
│ ├─ SR (Short Record)                    │
│ ├─ IL (ID Length)                       │
│ └─ TNF (Type Name Format)               │
├─────────────────────────────────────────┤
│ Type Length (1 byte)                    │
├─────────────────────────────────────────┤
│ Payload Length (1-3 bytes)              │
├─────────────────────────────────────────┤
│ ID Length (1 byte, ha IL=1)            │
├─────────────────────────────────────────┤
│ Type (Type Length bytes)                │
├─────────────────────────────────────────┤
│ ID (ID Length bytes, ha IL=1)           │
├─────────────────────────────────────────┤
│ Payload (Payload Length bytes)          │
└─────────────────────────────────────────┘
```

### NDEF URI Record (URL esetén)

**Header:** `0xD1` = `1101 0001`
- MB=1 (Message Begin)
- ME=1 (Message End)
- SR=1 (Short Record - 1 byte payload length)
- TNF=001 (Well Known Type)

**Type:** `0x55` = "U" (URI Record)

**Payload:** `[Prefix Code][URI...]`

**URI Prefix Codes:**
- `0x00` = Nincs prefix
- `0x01` = `http://www.`
- `0x02` = `https://www.`
- `0x03` = `http://`
- `0x04` = `https://`
- `0x05` = `tel:`
- `0x06` = `mailto:`
- stb.

### Példa NDEF Üzenet

**URL:** `https://example.com`

**NDEF Record:**
```
D1 01 0F 55 04 65 78 61 6D 70 6C 65 2E 63 6F 6D
│  │  │  │  │  └─────────────────────────────┐
│  │  │  │  │                                │
│  │  │  │  └─ Prefix: 0x04 = "https://"    │
│  │  │  └─ Type: 0x55 = URI Record          │
│  │  └─ Payload Length: 15 bytes           │
│  └─ Type Length: 1 byte                   │
└─ Header: 0xD1                             │
                                            │
                                            └─ "example.com" (ASCII)
```

**TLV Formátumban a címkén:**
```
Block 4: [03 11 D1 01]  ← TLV Tag (0x03=NDEF), Length (17 bytes)
Block 5: [0F 55 04 65]  ← Payload Length, Type, Prefix, "e"
Block 6: [78 61 6D 70]  ← "xamp"
Block 7: [6C 65 2E 63]  ← "le.c"
Block 8: [6F 6D FE 00]  ← "om" + Terminator (0xFE)
```

---

## APDU Parancsok

Az alkalmazás **APDU (Application Protocol Data Unit)** parancsokat használ.

### READ Block Parancs

**APDU:** `FF B0 00 [BLOCK] 04`

- `FF` = CLA (Class) - PC/SC extended
- `B0` = INS (Instruction) - READ BINARY
- `00` = P1 (Parameter 1)
- `[BLOCK]` = P2 (Parameter 2) - Block száma (0-134)
- `04` = Le (Expected Length) - 4 bytes

**Válasz:**
```
[Block Data (4 bytes)] [90 00]
                        └─┬─┘
                          └─ Status: Success
```

**Példa:** Block 4 olvasása
```
Küldés:  FF B0 00 04 04
Válasz:  [03 11 D1 01] [90 00]
```

### WRITE Block Parancs

**APDU:** `FF D6 00 [BLOCK] 04 [DATA (4 bytes)]`

- `FF` = CLA
- `D6` = INS (Instruction) - UPDATE BINARY
- `00` = P1
- `[BLOCK]` = P2 - Block száma
- `04` = Lc (Command Data Length)
- `[DATA]` = 4 bytes adat

**Válasz:**
```
[90 00]  ← Success
```

**Példa:** Block 4 írása
```
Küldés:  FF D6 00 04 04 03 11 D1 01
Válasz:  [90 00]
```

---

## Mit Lehet Megtenni

### ✅ Jelenleg Implementálva

1. **URL írása NTAG216 címkére**
   - NDEF formátumban
   - Automatikus prefix kezelés
   - Maximum ~250 karakter URL

2. **URL olvasása NTAG216 címkéből**
   - NDEF parse-olás
   - Automatikus prefix visszaállítás
   - URL validáció

3. **NTAG216 típus ellenőrzés**
   - Capability Container ellenőrzés
   - Csak NTAG216 címkék elfogadása

4. **NFC olvasó detektálás**
   - PC/SC olvasók listázása
   - Csatlakoztatott eszközök ellenőrzése

### 🔧 További Lehetőségek (még nincs implementálva)

#### 1. Szöveg Írása/Olvasása
```rust
// NDEF Text Record írása
write_ndef_text(card: &Card, text: &str, language: &str) -> Result<()>
```

#### 2. VCard (Kontakt) Írása/Olvasása
```rust
// NDEF MIME Record - vCard formátum
write_ndef_vcard(card: &Card, vcard: &str) -> Result<()>
```

#### 3. WiFi Konfiguráció
```rust
// WFA (WiFi Alliance) specifikus NDEF record
write_ndef_wifi(card: &Card, ssid: &str, password: &str, security: &str) -> Result<()>
```

#### 4. Raw Byte Írása/Olvasása
```rust
// Közvetlen memória írás (NDEF nélkül)
write_raw_bytes(card: &Card, block: u8, data: &[u8; 4]) -> Result<()>
read_raw_bytes(card: &Card, start_block: u8, count: u8) -> Result<Vec<u8>>
```

#### 5. Password Védelem
```rust
// Password beállítása
set_password(card: &Card, password: &[u8; 4]) -> Result<()>

// Password védett írás
write_with_password(card: &Card, block: u8, data: &[u8; 4], password: &[u8; 4]) -> Result<()>
```

#### 6. Read-Only Mód
```rust
// Címke read-only módba helyezése (visszafordíthatatlan!)
set_read_only(card: &Card) -> Result<()>
```

#### 7. NDEF Üzenet Törlése
```rust
// TLV terminátor írása (üres NDEF üzenet)
clear_ndef(card: &Card) -> Result<()>
```

#### 8. Többszörös NDEF Record-ok
```rust
// Több NDEF record egy üzenetben
write_ndef_message(card: &Card, records: Vec<NdefRecord>) -> Result<()>
```

---

## Implementáció Részletei

### 1. Címke Csatlakoztatás

```rust
// reader.rs
pub fn connect(&self) -> Result<Card> {
    // 1. Olvasók listázása
    let readers = self.ctx.list_readers()?;
    
    // 2. Első elérhető olvasó kiválasztása
    let reader = readers.first().unwrap();
    
    // 3. Címke csatlakoztatása (amikor az olvasóra helyezed)
    let card = self.ctx.connect(reader, ShareMode::Shared, Protocols::ANY)?;
    
    Ok(card)
}
```

### 2. NTAG216 Ellenőrzés

```rust
// ntag216.rs
pub fn check_type(&self, card: &Card) -> Result<bool> {
    // Block 3 olvasása (Capability Container)
    let cc = self.read_block(card, 3)?;
    
    // E1 = NDEF magic number (NTAG216 azonosító)
    Ok(cc[0] == 0xE1)
}
```

### 3. NDEF URL Írás Folyamata

```
1. NTAG216 ellenőrzés (check_type)
   ↓
2. URL → NDEF üzenet konverzió (create_ndef_url)
   ├─ Header byte generálás
   ├─ Prefix code meghatározás
   └─ Payload összeállítás
   ↓
3. TLV formátum létrehozása
   ├─ Tag: 0x03 (NDEF)
   ├─ Length: NDEF üzenet hossza
   └─ Value: NDEF üzenet
   ↓
4. Block-okra bontás (4 bytes/block)
   ↓
5. Block-ok írása (block 4-től kezdve)
   ↓
6. Terminátor írása (0xFE)
```

### 4. NDEF URL Olvasás Folyamata

```
1. NTAG216 ellenőrzés
   ↓
2. Block 4 olvasása (TLV header)
   ├─ Tag ellenőrzés (0x03 = NDEF)
   └─ Length meghatározás
   ↓
3. További block-ok olvasása (Length alapján)
   ↓
4. NDEF üzenet összeállítása
   ↓
5. NDEF parse-olás (parse_ndef_url)
   ├─ Header ellenőrzés
   ├─ Type ellenőrzés (0x55 = URI)
   ├─ Prefix code kinyerése
   └─ URL összeállítása
   ↓
6. Visszaadás
```

### 5. Hibakezelés

**PC/SC Hibák:**
- `Nincs NFC olvasó csatlakoztatva` - USB olvasó hiányzik
- `Nem sikerült csatlakozni` - Címke nincs az olvasó közelében

**APDU Hibák:**
- `SW1=0x63, SW2=0xCX` - Authentication failed (password szükséges)
- `SW1=0x6A, SW2=0x82` - File not found (érvénytelen block)
- `SW1=0x6A, SW2=0x86` - Wrong parameters

**NTAG216 Hibák:**
- `Ez nem egy NTAG216 címke` - CC[0] != 0xE1
- `Nem található NDEF üzenet` - Nincs TLV vagy üres
- `Az URL túl hosszú` - >250 karakter

---

## Használati Példák

### URL Írása

```rust
let reader = NfcReader::new()?;
let card = reader.connect()?;  // Helyezd a címkét az olvasóra!
let ntag = Ntag216;

ntag.write_ndef_url(&card, "https://example.com")?;
```

### URL Olvasása

```rust
let reader = NfcReader::new()?;
let card = reader.connect()?;  // Helyezd a címkét az olvasóra!
let ntag = Ntag216;

match ntag.read_ndef(&card)? {
    Some(url) => println!("Olvasott URL: {}", url),
    None => println!("Nincs NDEF üzenet"),
}
```

### Block Olvasása

```rust
let block_data = ntag.read_block(&card, 4)?;
println!("Block 4: {:02X?}", block_data);
```

### Block Írása

```rust
let data = [0x03, 0x11, 0xD1, 0x01];
ntag.write_block(&card, 4, &data)?;
```

---

## Korlátok és Megjegyzések

### Korlátok

1. **Kapacitás:** Maximum 504 bytes NDEF adat (126 blocks)
2. **URL hossz:** ~250 karakter (prefix code-okkal együtt)
3. **Olvasási távolság:** ~10 cm (olvasótól függően)
4. **Írási távolság:** ~5-7 cm (rövidebb, mint olvasás)
5. **Block méret:** Mindig 4 bytes (nem változtatható)

### Fontos Megjegyzések

⚠️ **Block 0-3 READ ONLY** - Nem írható!

⚠️ **Read-Only mód visszafordíthatatlan** - Ha beállítod, soha többé nem írható!

⚠️ **Password védelem** - PC/SC API-n keresztül korlátozottan működik

⚠️ **Block 130-134** - Konfigurációs blokkok, óvatosan kezeld!

---

## További Információk

- **NXP NTAG216 Datasheet:** [Hivatalos specifikáció](https://www.nxp.com/docs/en/data-sheet/NTAG216.pdf)
- **NDEF Specifikáció:** NFC Forum NDEF Technical Specification
- **PC/SC Standard:** ISO/IEC 7816

---

**Készítve:** NTAG216 NFC Alkalmazás  
**Verzió:** 1.0  
**Dátum:** 2024

