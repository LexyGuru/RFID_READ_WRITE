# Tauri NFC Alkalmazás - NTAG216 és Mifare Classic támogatás

## Áttekintés

Ez a projekt egy Tauri alapú asztali alkalmazás, amely NFC olvasót kezel és képes írni/olvasni **NTAG216** és **Mifare Classic** címkéket.

## Fontos: Parancsok használata

⚠️ **FIGYELEM**: A projekt Tauri v1-re van beállítva. Használd az **npm scriptet**, ne a `cargo tauri` parancsot közvetlenül!

### ✅ Helyes parancsok:

```bash
# Fejlesztési mód indítása
npm run tauri dev

# Production build
npm run tauri build
```

### ❌ Ne használd ezt:

```bash
# Ez NEM működik, mert a Tauri v2 CLI-t hívja meg
cargo tauri dev
```

## NFC Címke Típusok

### NTAG216
- **Típus:** NFC Type 2 Tag (ISO14443A)
- **Kapacitás:** 888 bytes (felhasználói adat)
- **Frekvencia:** 13.56 MHz
- **Olvasási távolság:** ~10 cm
- **Biztonság:** Alapvető védelem, jelszóval védhető

**Használati esetek:**
- ✅ Egyszerű adattárolás (URL-ek, szövegek, kontaktok)
- ✅ NDEF formátumú üzenetek
- ✅ Marketing kampányok (QR kód alternatíva)
- ✅ Egyszerű belépési rendszerek
- ✅ Termék információ tárolás
- ✅ IoT eszköz konfiguráció

**Előnyök:**
- Olcsó és széleskörűen elérhető
- Könnyen programozható
- NDEF standard támogatás
- Kompatibilis legtöbb NFC olvasóval

### Mifare Classic
- **Típus:** ISO14443A
- **Kapacitás:** 1KB (Mifare Classic 1K) vagy 4KB (Mifare Classic 4K)
- **Frekvencia:** 13.56 MHz
- **Olvasási távolság:** ~10 cm
- **Biztonság:** Crypto1 titkosítás, kulcsokkal védett szektorok

**Használati esetek:**
- ✅ Közlekedési kártyák (BKK, MÁV)
- ✅ Belépési rendszerek
- ✅ Fizetési kártyák (régebbi rendszerek)
- ✅ Szektor-alapú adattárolás
- ✅ Biztonságos adattárolás kulcsokkal
- ✅ Többszintű hozzáférés-vezérlés

**Előnyök:**
- Szektor-alapú szervezés
- Kulcsokkal védett szektorok
- Nagyobb kapacitás (4K verzió)
- Széleskörűen használt (közlekedés, belépés)

**Korlátok:**
- Crypto1 titkosítás sebezhető (de még mindig használatos)
- Komplexebb programozás
- NDEF nem natív támogatás

## Technikai Megközelítések

### 1. Tauri NFC Plugin (Mobil - Android/iOS)
A Tauri v2 rendelkezik beépített NFC pluginnal, de ez **csak mobil platformokon** működik:
- Android
- iOS

**Korlátok:**
- Csak NDEF formátumot támogat
- NTAG216: ✅ Teljes támogatás
- Mifare Classic: ⚠️ Korlátozott (csak NDEF, ha van)

### 2. Desktop Megoldások (PC/Mac/Linux)

Asztali platformokon nincs natív NFC támogatás a Tauri-ban. Lehetőségek:

#### A) PC/SC (Smart Card) API
- **Windows:** WinSCard API
- **Linux:** PC/SC Lite
- **macOS:** PC/SC framework

#### B) Rust NFC Könyvtárak
- `pcsc` - PC/SC wrapper Rust-ban
- `nfc-rs` - Alacsony szintű NFC hozzáférés
- `libnfc` binding-ek

#### C) USB NFC Olvasók
- ACR122U (ACS)
- PN532 modulok
- Legic Prime modulok

## Projekt Struktúra

```
nfc-rust/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Tauri entry point
│   │   ├── nfc/
│   │   │   ├── mod.rs       # NFC modul
│   │   │   ├── ntag216.rs   # NTAG216 kezelés
│   │   │   └── reader.rs    # PC/SC olvasó kezelés
│   │   └── commands.rs      # Tauri command-ok
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                      # Frontend (HTML/JS)
│   └── index.html
├── package.json
└── vite.config.js
```

## Telepítés és Futtatás

### Előfeltételek

1. **Rust telepítés:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js telepítés:**
   - [Node.js letöltés](https://nodejs.org/)

3. **PC/SC szolgáltatás:**
   - **macOS:** Beépített (nincs külön telepítés)
   - **Linux:** `sudo apt-get install pcscd libpcsclite1`
   - **Windows:** Automatikusan telepítve

### Projekt Telepítés

```bash
# Függőségek telepítése
npm install

# Rust függőségek letöltése
cd src-tauri
cargo build
cd ..
```

### Futtatás

```bash
# Fejlesztési mód (használd ezt!)
npm run tauri dev

# Production build
npm run tauri build
```

## Használat

1. **NFC olvasó csatlakoztatása**
   - Csatlakoztasd az USB NFC olvasót a számítógéphez
   - Az alkalmazás automatikusan felismeri

2. **URL írása NTAG216 címkére**
   - Add meg az URL-t a beviteli mezőben
   - Kattints az "URL írása NTAG216 címkére" gombra
   - Olvass be egy NTAG216 címkét az olvasóra
   - Várd meg a sikeres írás üzenetet

3. **URL olvasása és automatikus megnyitás**
   - Kattints az "URL olvasása NTAG216 címkéből" gombra
   - Olvass be egy NTAG216 címkét az olvasóra
   - Az URL automatikusan megnyílik a böngészőben

## Implementációs Lehetőségek

### NTAG216 Írás/Olvasás
1. **NDEF üzenetek** - Standard NFC formátum
2. **Raw byte írás** - Közvetlen memória írás
3. **Password védelem** - Jelszóval védett írás

### Mifare Classic Írás/Olvasás
1. **Szektor olvasás/írás** - Kulcsokkal védett szektorok
2. **Block írás** - 16 byte-os blokkok
3. **Kulcs kezelés** - A/B kulcsok kezelése
4. **Value block műveletek** - Növelés/csökkentés műveletek

## Hibakeresés

### "Nincs NFC olvasó csatlakoztatva"
- Ellenőrizd, hogy az USB olvasó csatlakoztatva van-e
- Linux: `pcsc_scan` parancs futtatása
- Windows: Eszközkezelőben ellenőrizd

### "Card connect error"
- A címke nincs az olvasó közelében
- Próbáld meg újra

### "Write failed" vagy "Read failed"
- A címke lehet, hogy read-only módban van
- Ellenőrizd, hogy NTAG216 címkét használsz-e

## CI/CD - GitHub Actions

A projekt automatikusan fordít mindhárom platformra (Windows, Linux, macOS) GitHub Actions segítségével.

### Automatikus Build

Minden push a `main` vagy `master` branch-re automatikusan elindítja a build folyamatot:
- **macOS:** x86_64 és aarch64 (Apple Silicon) verziók
- **Linux:** AppImage formátum
- **Windows:** MSI installer

### Release-ek

A sikeres build után automatikusan létrejön egy GitHub Release draft verzió, amely tartalmazza az összes platform build fájljait.

### Workflow fájl

A build konfiguráció a `.github/workflows/build.yml` fájlban található.

## További Információk

- `COMPARISON.md` - Részletes összehasonlítás NTAG216 vs Mifare Classic
- `IMPLEMENTATION_GUIDE.md` - Implementációs útmutató kóddal
- `OSSZEFOGLALO.md` - Rövid összefoglaló válaszokkal
- `INSTALLATION.md` - Telepítési útmutató
