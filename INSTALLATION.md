# Telepítési Útmutató

## Előfeltételek

### 1. Rust Telepítés
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Node.js és npm
Telepítsd a Node.js-t (v16 vagy újabb):
- [Node.js letöltés](https://nodejs.org/)

### 3. PC/SC Szolgáltatás (Desktop platformokhoz)

#### macOS
```bash
# PC/SC framework beépített macOS-ben
# Nincs külön telepítés szükséges
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install pcscd libpcsclite1 libpcsclite-dev
sudo systemctl enable pcscd
sudo systemctl start pcscd
```

#### Windows
- A Windows Smart Card szolgáltatás automatikusan telepítve van
- Ellenőrizd, hogy fut-e: `services.msc` → "Smart Card" szolgáltatás

### 4. NFC Olvasó
Szükséged lesz egy USB NFC olvasóra:
- **ACR122U** (ACS) - Ajánlott
- **PN532** modulok
- Bármilyen PC/SC kompatibilis NFC olvasó

## Projekt Telepítés

### 1. Függőségek telepítése
```bash
cd "NFC RUST"
npm install
```

### 2. Rust függőségek letöltése
```bash
cd src-tauri
cargo build
cd ..
```

## Futtatás

### Fejlesztési mód
```bash
npm run tauri dev
```

### Production build
```bash
npm run tauri build
```

A build eredménye a `src-tauri/target/release/` mappában lesz.

## Használat

1. **NFC olvasó csatlakoztatása**
   - Csatlakoztasd az USB NFC olvasót a számítógéphez
   - Ellenőrizd, hogy az alkalmazás felismeri-e

2. **URL írása**
   - Add meg az URL-t a beviteli mezőben
   - Kattints az "URL írása NTAG216 címkére" gombra
   - Olvass be egy NTAG216 címkét az olvasóra
   - Várd meg a sikeres írás üzenetet

3. **URL olvasása**
   - Kattints az "URL olvasása NTAG216 címkéből" gombra
   - Olvass be egy NTAG216 címkét az olvasóra
   - Az URL automatikusan megnyílik a böngészőben

## Hibaelhárítás

### "Nincs NFC olvasó csatlakoztatva"
- Ellenőrizd, hogy az USB olvasó csatlakoztatva van-e
- Linux: `pcsc_scan` parancs futtatása az olvasó ellenőrzéséhez
- Windows: Eszközkezelőben ellenőrizd az olvasót

### "Card connect error"
- A címke nincs az olvasó közelében
- Próbáld meg újra, és győződj meg róla, hogy a címke az olvasón van

### "Write failed" vagy "Read failed"
- A címke lehet, hogy read-only módban van
- Ellenőrizd, hogy NTAG216 címkét használsz-e (nem Mifare Classic)
- Próbáld meg egy másik címkével

### PC/SC szolgáltatás nem fut (Linux)
```bash
sudo systemctl status pcscd
sudo systemctl start pcscd
```

## Platform Specifikus Megjegyzések

### macOS
- Először lehet, hogy engedélyt kell adnod az alkalmazásnak a Smart Card hozzáféréshez
- Rendszerbeállítások → Biztonság és adatvédelem → Smart Card

### Linux
- Lehet, hogy felhasználói jogosultságok szükségesek
- Hozzáadd magad a `pcscd` csoporthoz: `sudo usermod -a -G pcscd $USER`
- Újra be kell jelentkezned a változások érvényesítéséhez

### Windows
- A Smart Card szolgáltatás automatikusan fut
- Ha problémák vannak, indítsd újra a szolgáltatást: `services.msc`



