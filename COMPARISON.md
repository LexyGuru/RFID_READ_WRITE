# NTAG216 vs Mifare Classic - Részletes Összehasonlítás

## Technikai Specifikációk

| Tulajdonság | NTAG216 | Mifare Classic 1K | Mifare Classic 4K |
|------------|---------|-------------------|-------------------|
| **Kapacitás** | 888 bytes | 1 KB (1024 bytes) | 4 KB (4096 bytes) |
| **Szektorok** | N/A (lineáris) | 16 szektor | 40 szektor |
| **Blokkok** | 135 blokk | 64 blokk | 256 blokk |
| **Blokk méret** | 4 bytes | 16 bytes | 16 bytes |
| **Titkosítás** | Password védelem | Crypto1 | Crypto1 |
| **NDEF támogatás** | ✅ Natív | ⚠️ Korlátozott | ⚠️ Korlátozott |
| **ISO szabvány** | ISO14443A Type 2 | ISO14443A Type A | ISO14443A Type A |
| **UID méret** | 7 bytes | 4 bytes | 4 bytes |
| **Ár** | Olcsó (~100-500 Ft) | Közepes (~500-2000 Ft) | Drágább (~1000-3000 Ft) |

## Használati Esetek Részletesen

### NTAG216 - Ideális Használatok

#### 1. **Marketing és Promóció**
- **QR kód helyettesítés** - Közvetlenül megnyitja az URL-t
- **Social media linkek** - Instagram, Facebook profilok
- **Termék információk** - Ár, leírás, készlet
- **Esemény információk** - Konferenciák, koncertek

**Példa:**
```
NDEF Record: URI Record
https://example.com/product/12345
```

#### 2. **Egyszerű Belépési Rendszerek**
- Irodai belépés
- Szobák/terem hozzáférés
- Parkoló belépés
- Egyszerű időmérés

**Implementáció:**
- UID alapú azonosítás
- Password védelem opcionális

#### 3. **IoT Eszköz Konfiguráció**
- WiFi hitelesítő adatok
- Eszköz párosítás
- Gyári beállítások visszaállítása

**Példa:**
```
NDEF Record: WiFi Record
SSID: MyNetwork
Password: MyPassword123
Security: WPA2
```

#### 4. **Kontakt Információk**
- VCard formátum
- Telefonszámok
- Email címek
- Címek

#### 5. **Egyszerű Adattárolás**
- Szöveges jegyzetek
- Rendezvények
- Feladatok listája

### Mifare Classic - Ideális Használatok

#### 1. **Közlekedési Kártyák**
- **BKK Budapest kártya** - Jegyek, bérletek
- **MÁV kártya** - Vonatjegyek
- **Helyi közlekedés** - Busz, villamos, metró

**Implementáció:**
- Szektor-alapú érték tárolás
- Value block műveletek (növelés/csökkentés)
- Időbélyeg tárolás
- Zóna információk

#### 2. **Belépési Rendszerek**
- **Irodaházak** - Szint, szoba hozzáférés
- **Hotel szobák** - Kulcs kártyák
- **Társasházak** - Kapu, lift hozzáférés
- **Parkolók** - Időalapú hozzáférés

**Szektor struktúra példa:**
```
Szektor 0: UID, gyártási adatok (read-only)
Szektor 1-14: Felhasználói adatok
Szektor 15: Master kulcsok
```

#### 3. **Fizetési Rendszerek**
- **Kantin kártyák** - Előfizetéses rendszerek
- **Büfé kártyák** - Pontgyűjtő rendszerek
- **Parkoló kártyák** - Időalapú fizetés

**Value Block műveletek:**
- `increment(value)` - Érték növelés
- `decrement(value)` - Érték csökkentés
- `transfer()` - Érték átvitel

#### 4. **Biztonságos Adattárolás**
- **Kulcs tárolás** - Kulcsok, kódok
- **Személyes adatok** - Titkosított információk
- **Log fájlok** - Esemény naplózás

**Szektor védelem:**
```
Key A: Olvasás/írás kulcs
Key B: Írás kulcs vagy adat
Access Bits: Hozzáférési jogosultságok
```

#### 5. **Többszintű Hozzáférés**
- **Szint 1:** Olvasás mindenki számára
- **Szint 2:** Írás Key A-val
- **Szint 3:** Csak Key B-vel írható
- **Szint 4:** Teljes zárolás

## Programozási Különbségek

### NTAG216 Programozás

```rust
// NDEF üzenet írása
let ndef_message = vec![
    NdefRecord::Uri("https://example.com".to_string()),
    NdefRecord::Text("Hello World".to_string()),
];

// Password beállítás
ntag216.set_password(0x00000000)?;
ntag216.enable_password_protection()?;

// Raw byte írás
ntag216.write_bytes(0x04, &data)?;
```

**Előnyök:**
- Egyszerű API
- Standard NDEF formátum
- Nincs kulcs kezelés
- Lineáris memória

### Mifare Classic Programozás

```rust
// Szektor olvasás Key A-val
let key_a = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
mifare.authenticate_sector(sector, &key_a, KeyType::A)?;
let data = mifare.read_block(block)?;

// Szektor írás
mifare.write_block(block, &data)?;

// Value block művelet
mifare.increment_value(block, 10)?;
mifare.transfer_value(block)?;

// Access bits módosítás
mifare.update_access_bits(sector, &access_bits)?;
```

**Előnyök:**
- Szektor-alapú szervezés
- Kulcs-alapú biztonság
- Value block műveletek
- Rugalmas hozzáférés-vezérlés

**Hátrányok:**
- Komplexebb API
- Kulcs kezelés szükséges
- Access bits konfiguráció
- Crypto1 sebezhetőség

## Biztonsági Megfontolások

### NTAG216 Biztonság
- ✅ Password védelem (32-bit)
- ✅ Read-only zóna beállítható
- ⚠️ Nincs titkosítás
- ⚠️ UID klónozható

**Ajánlások:**
- Server-side validáció
- UID + timestamp kombináció
- Password védelem használata

### Mifare Classic Biztonság
- ✅ Crypto1 titkosítás
- ✅ Kulcs-alapú védelem
- ✅ Access bits vezérlés
- ⚠️ Crypto1 sebezhető (de még használatos)
- ⚠️ UID klónozható

**Ajánlások:**
- Egyedi kulcsok használata
- Server-side validáció
- Időbélyeg ellenőrzés
- Szektor-alapú hozzáférés

## Döntési Segédlet

### Válaszd az NTAG216-ot, ha:
- ✅ Egyszerű adattárolás kell
- ✅ NDEF formátumot használsz
- ✅ Marketing/promóció cél
- ✅ Olcsó megoldás kell
- ✅ Könnyű programozás fontos
- ✅ IoT konfiguráció

### Válaszd a Mifare Classic-ot, ha:
- ✅ Szektor-alapú szervezés kell
- ✅ Kulcs-alapú biztonság fontos
- ✅ Value block műveletek kellenek
- ✅ Többszintű hozzáférés szükséges
- ✅ Kompatibilitás meglévő rendszerekkel
- ✅ Nagyobb kapacitás kell (4K)

## Kompatibilitás

### Olvasók Támogatása
- **ACR122U:** ✅ Mindkettőt támogatja
- **PN532:** ✅ Mindkettőt támogatja
- **Mobil NFC:** ✅ NTAG216 teljes, Mifare Classic korlátozott
- **PC/SC olvasók:** ✅ Mindkettőt támogatja

### Platform Támogatás
- **Windows:** ✅ PC/SC API
- **Linux:** ✅ PC/SC Lite, libnfc
- **macOS:** ✅ PC/SC framework
- **Android:** ✅ Tauri NFC plugin
- **iOS:** ✅ Tauri NFC plugin (NDEF csak)

## Összefoglalás

**NTAG216:** Egyszerű, olcsó, széleskörűen támogatott, ideális marketing és egyszerű alkalmazásokhoz.

**Mifare Classic:** Komplexebb, szektor-alapú, kulcs-alapú biztonság, ideális közlekedési és belépési rendszerekhez.



