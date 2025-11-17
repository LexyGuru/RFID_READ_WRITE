# NTAG216 Gyors Összefoglaló

## 🎯 Mi az NTAG216?

Egy **passzív NFC címke** (nincs saját áramforrása), amely:
- 📱 **888 bytes** adat tárolására képes
- 🔄 **13.56 MHz** frekvencián kommunikál
- 📏 **~10 cm** olvasási távolság
- ✅ **NDEF formátum** natív támogatás

---

## 🔧 Hogyan Működik?

```
1. NFC Olvasó → RF jel küldése (13.56 MHz)
2. NTAG216 → Energiát kap, aktiválódik
3. Olvasó → APDU parancs küldése
4. NTAG216 → Válasz küldése
5. Adatok → Olvasó → Alkalmazás
```

**Kommunikáció:** PC/SC API → USB NFC Olvasó → NTAG216

---

## 📊 Memória Struktúra

```
┌─────────────────────────────┐
│ Block 0-3:   UID (READ ONLY) │  16 bytes
├─────────────────────────────┤
│ Block 4-129: User Data       │  504 bytes ← ITT TÁROLÓDIK AZ NDEF
├─────────────────────────────┤
│ Block 130-134: Config        │  20 bytes
└─────────────────────────────┘

ÖSSZESEN: 135 blocks × 4 bytes = 540 bytes
```

---

## ✅ Mit Lehet Megtenni?

### Jelenleg Implementálva:

1. **📝 URL írása** - NDEF formátumban
2. **📖 URL olvasása** - NDEF parse-olás
3. **🔍 Típus ellenőrzés** - NTAG216 azonosítás
4. **📡 Olvasó detektálás** - PC/SC olvasók listázása

### További Lehetőségek (lehet implementálni):

- 📄 **Szöveg írása/olvasása** (NDEF Text Record)
- 👤 **VCard írása** (Kontakt információk)
- 📶 **WiFi konfiguráció** (WFA specifikus)
- 🔒 **Password védelem** beállítása
- 🔐 **Read-only mód** (visszafordíthatatlan!)
- 🗑️ **NDEF törlése**
- 📦 **Többszörös NDEF record-ok**

---

## 📋 NDEF URL Formátum

**Példa:** `https://example.com`

```
TLV Formátum:
[03] [11] [D1 01 0F 55 04 65 78 61 6D 70 6C 65 2E 63 6F 6D] [FE]
 │    │    │  │  │  │  └─────────────────────────────────┐ │
 │    │    │  │  │  │                                    │ │
 │    │    │  │  │  └─ Prefix: 0x04 = "https://"        │ │
 │    │    │  │  └─ Type: 0x55 = URI Record              │ │
 │    │    │  └─ Payload Length: 15 bytes                │ │
 │    │    └─ Header: 0xD1 (MB=1, ME=1, SR=1, TNF=001)   │ │
 │    └─ Length: 17 bytes                                │ │
 └─ Tag: 0x03 = NDEF                                    │ │
                                                        │ │
                                                        └─ "example.com"
                                                          └─ Terminator
```

**Prefix Codes:**
- `0x01` = `http://www.`
- `0x02` = `https://www.`
- `0x03` = `http://`
- `0x04` = `https://`

---

## 🔌 APDU Parancsok

### READ Block
```
Küldés:  FF B0 00 [BLOCK] 04
Válasz:  [DATA (4 bytes)] [90 00]
```

### WRITE Block
```
Küldés:  FF D6 00 [BLOCK] 04 [DATA (4 bytes)]
Válasz:  [90 00]
```

---

## 💻 Kód Példák

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

let url = ntag.read_ndef(&card)?;
println!("URL: {:?}", url);
```

### Block Olvasása
```rust
let block = ntag.read_block(&card, 4)?;
println!("Block 4: {:02X?}", block);
```

---

## ⚠️ Fontos Megjegyzések

1. **Block 0-3 READ ONLY** - Nem írható!
2. **Read-only mód visszafordíthatatlan** - Óvatosan!
3. **Olvasási távolság:** ~10 cm
4. **Írási távolság:** ~5-7 cm (rövidebb)
5. **Maximum URL hossz:** ~250 karakter

---

## 🐛 Gyakori Hibák

| Hiba | Ok | Megoldás |
|------|-----|----------|
| `Nincs NFC olvasó` | USB olvasó nincs csatlakoztatva | Csatlakoztasd az olvasót |
| `Nem sikerült csatlakozni` | Címke nincs az olvasó közelében | Helyezd a címkét az olvasóra |
| `Ez nem egy NTAG216` | Más típusú címke | Használj NTAG216 címkét |
| `Nem található NDEF` | Nincs NDEF üzenet a címkén | Írj először NDEF üzenetet |

---

## 📚 További Információ

Részletes dokumentáció: `NTAG216_DOKUMENTACIO.md`

**Specifikációk:**
- NXP NTAG216 Datasheet
- NFC Forum NDEF Specification
- ISO/IEC 7816 (PC/SC)





