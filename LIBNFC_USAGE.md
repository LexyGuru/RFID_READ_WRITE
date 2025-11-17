# libnfc Használat

## Telepítés

Lásd: [`LIBNFC_SETUP.md`](LIBNFC_SETUP.md)

## Build libnfc feature-rel

```bash
cd src-tauri
cargo build --features libnfc --no-default-features
```

Vagy mindkettőt használva:

```bash
cargo build --features "pcsc,libnfc"
```

## Használat

A libnfc feature engedélyezésekor az alkalmazás automatikusan libnfc-et használ PC/SC helyett.

### Előnyök libnfc használatával

- ✅ **Jobb password authentication támogatás**: Az NTAG216 password authentication (PWD_AUTH) működik libnfc-n keresztül
- ✅ **Natív NFC támogatás**: Közvetlenül az NFC olvasóval kommunikál, nem PC/SC API-n keresztül
- ✅ **Több olvasó támogatás**: Többféle NFC olvasót támogat

### Hátrányok

- ⚠️ **Telepítés szükséges**: libnfc-et külön kell telepíteni
- ⚠️ **Konfiguráció szükséges**: libnfc konfigurációs fájlt kell beállítani
- ⚠️ **PC/SC ütközés**: Lehet, hogy le kell állítani a PC/SC démont

## Konfiguráció

Az ACR122U-nál a következő konfiguráció működhet:

```bash
sudo nano /usr/local/etc/nfc/libnfc.conf
```

Vagy:

```bash
sudo nano /opt/homebrew/etc/nfc/libnfc.conf
```

Add hozzá:

```
device.name = "ACR122U"
device.connstring = "acr122_usb:"
```

Vagy:

```
device.name = "ACR122U"
device.connstring = "acr122_pcsc:"
```

## Tesztelés

```bash
nfc-list
```

Ha működik, látnod kellene az NFC olvasót és az esetlegesen észlelt címkéket.




