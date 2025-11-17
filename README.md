# NTAG216 NFC Alkalmazás

Egy Tauri alapú asztali alkalmazás, amely kizárólag **NTAG216** NFC címkéket támogat.

## Áttekintés

Ez az alkalmazás PC/SC API-t használ az NFC olvasó kommunikációjához, és NDEF formátumban ír/olvas URL-eket NTAG216 címkéken.

### NTAG216 Specifikáció

- **Típus:** NFC Type 2 Tag (ISO14443A)
- **Kapacitás:** 888 bytes felhasználói adat
- **Blokkok:** 135 blocks (4 bytes/block)
- **Frekvencia:** 13.56 MHz
- **Olvasási távolság:** ~10 cm

## Előfeltételek

### macOS
- **PC/SC framework:** Beépített macOS-ben (nincs külön telepítés szükséges)
- **ACS CCID Driver:** Ha ACS olvasót használsz (pl. ACR122U, ACR1251U), telepítsd az [ACS CCID Driver-t](https://www.acs.com.hk/en/driver/3/acr122u-usb-nfc-reader/) macOS-re
  - A natív driver telepítése után az olvasó felismerhető lesz a PC/SC-n keresztül
  - **Fontos:** A PC/SC API korlátai miatt bizonyos műveletek (pl. password védelem konfiguráció blokkok írása) nem mindig működnek még natív driverrel sem

### Linux
```bash
sudo apt-get install pcscd libpcsclite1
sudo systemctl start pcscd
```

### Windows
- PC/SC automatikusan telepítve

## Telepítés

```bash
# Függőségek telepítése
npm install

# Rust függőségek letöltése
cd src-tauri
cargo build
cd ..
```

## Futtatás

```bash
# Fejlesztési mód
npm run tauri dev

# Production build
npm run tauri build
```

## Használat

1. **NFC olvasó csatlakoztatása**
   - Csatlakoztasd az USB NFC olvasót a számítógéphez
   - Kattints az "Olvasók ellenőrzése" gombra

2. **URL írása NTAG216 címkére**
   - Add meg az URL-t a beviteli mezőben
   - Kattints az "URL írása NTAG216 címkére" gombra
   - Helyezd az NTAG216 címkét az olvasóra
   - Várd meg a sikeres írás üzenetet

3. **URL olvasása NTAG216 címkéből**
   - Kattints az "URL olvasása NTAG216 címkéből" gombra
   - Helyezd az NTAG216 címkét az olvasóra
   - Az URL automatikusan megnyílik a böngészőben

## Projekt Struktúra

```
nfc-rust/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Tauri entry point
│   │   └── nfc/
│   │       ├── mod.rs       # NFC modul
│   │       ├── ntag216.rs   # NTAG216 specifikus implementáció
│   │       └── reader.rs    # PC/SC olvasó kezelés
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html               # Frontend UI
├── package.json
└── vite.config.js
```

## Technikai Részletek

### NDEF Formátum

Az alkalmazás NDEF (NFC Data Exchange Format) formátumot használ az URL-ek tárolásához:
- **Record Type:** Well Known Type (TNF=0x01)
- **Type:** URI Record (U=0x55)
- **Payload:** Prefix code + URL

### PC/SC API

- **APDU parancsok:** READ (0xFF 0xB0) és WRITE (0xFF 0xD6)
- **Block méret:** 4 bytes
- **User data:** Block 4-129 (126 blocks = 504 bytes)

#### PC/SC API Korlátok

⚠️ **Fontos:** A PC/SC API egy standardizált réteg, ami nem minden natív NFC funkciót támogat teljesen:

- ✅ **Működik:** Block olvasás/írás (4-129), NDEF üzenetek írása/olvasása
- ⚠️ **Korlátozottan működik:** Password védelem konfiguráció (Block 130-134)
  - Block 130 (Password) írása általában működik
  - Block 131-133 (PACK, ACCESS, Auth Limit) írása password beállítás után nem mindig működik PC/SC API-n keresztül
  - **Password authentication (PWD_AUTH) nem működik az ACR122U-nál PC/SC API-n keresztül**
  - Ez **nem az olvasó driver hibája**, hanem a PC/SC API standard korlátja
- 💡 **Megoldás:** 
  - A password beállítása (Block 130) általában elég a védelem aktiválásához, a többi blokk opcionális
  - **Fontos:** Az ACR122U-nál password védelemmel védett címkéket csak password nélkül lehet írni/olvasni PC/SC API-n keresztül
  - Password authentication működéséhez más NFC olvasó szükséges lehet

**Miért van ez?**
- A PC/SC API-t eredetileg smart card-okhoz tervezték, nem NFC címkékhez
- Az NTAG216 password authentication speciális művelet, ami nem mindig illeszkedik a PC/SC standardhoz
- Még az ACS CCID natív driver telepítése után is ezek a korlátok fennállhatnak

## Hibakeresés

### "Nincs NFC olvasó csatlakoztatva"
- Ellenőrizd, hogy az USB olvasó csatlakoztatva van-e
- **macOS + ACS olvasó:** Telepítsd az ACS CCID Driver-t
- Linux: `pcsc_scan` parancs futtatása
- macOS: Rendszerbeállítások > Biztonság és adatvédelem

### "Password védelem konfiguráció blokkok nem írhatók"
- ⚠️ Ez egy ismert PC/SC API korlát
- A Block 130 (Password) írása általában működik
- A Block 131-133 írása password után nem mindig működik PC/SC API-n keresztül
- **Ez nem az olvasó vagy driver hibája**, hanem a PC/SC API standard korlátja
- A password beállítása (Block 130) általában elég a védelem aktiválásához

### "Password authentication sikertelen (SW1=0x63, SW2=0x00)"
- ⚠️ **Ismert korlát az ACR122U-nál**
- Az ACR122U-nál az NTAG216 password authentication (PWD_AUTH) **nem működik PC/SC API-n keresztül**
- Ez nem az olvasó vagy driver hibája, hanem a PC/SC API standard korlátja
- **Megoldások:**
  - Próbáld meg password nélkül írni/olvasni (ha lehet)
  - Használj más NFC olvasót, ami támogatja az NTAG216 password authentication-t
  - Vagy használj natív NFC driver-t (nem PC/SC API)

### "Ez nem egy NTAG216 címke!"
- Győződj meg róla, hogy NTAG216 címkét használsz
- Más NFC típusok (pl. Mifare Classic) nem támogatottak

### "Csatlakozási hiba"
- A címke nincs az olvasó közelében
- Próbáld meg újra, és biztosítsd, hogy a címke az olvasóra van helyezve

## Dokumentáció

📚 **Részletes dokumentáció:** [`NTAG216_DOKUMENTACIO.md`](NTAG216_DOKUMENTACIO.md)
- Teljes működési leírás
- Technikai specifikációk
- APDU parancsok részletei
- Implementáció részletei
- További lehetőségek

📋 **Gyors összefoglaló:** [`NTAG216_OSSZEFOGLALO.md`](NTAG216_OSSZEFOGLALO.md)
- Gyors áttekintés
- Főbb jellemzők
- Kód példák
- Gyakori hibák

🔐 **Configuration blokkok útmutató:** [`CONFIGURATION_BLOKKOK.md`](CONFIGURATION_BLOKKOK.md)
- Block 130-134 részletes magyarázata
- Password védelem beállítása
- Read-only mód
- Vizuális ábrák

## Licenc

MIT
