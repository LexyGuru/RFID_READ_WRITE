# libnfc Telepítés és Használat

## Telepítés macOS-en

### 1. Homebrew telepítése (ha még nincs)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### 2. libnfc telepítése

```bash
brew install libnfc
```

### 3. NFC olvasó csatlakoztatása és azonosítása

Csatlakoztasd az NFC olvasót a Mac számítógépéhez USB-n keresztül.

Listázd az elérhető USB-soros portokat:

```bash
ls /dev/tty.usb*
```

Jegyezd fel a megfelelő port nevét (pl. `/dev/tty.usbmodem14201`).

### 4. libnfc konfigurálása

Szerkeszd a libnfc konfigurációs fájlt:

```bash
sudo nano /usr/local/etc/nfc/libnfc.conf
```

Vagy ha a Homebrew más helyre telepítette:

```bash
sudo nano /opt/homebrew/etc/nfc/libnfc.conf
```

Add hozzá vagy módosítsd a következő sorokat a korábban megjegyzett port nevével:

```
device.name = "ACR122U"
device.connstring = "acr122_usb:"
```

Vagy ha USB-soros portot használsz:

```
device.name = "ACR122U"
device.connstring = "pn532_uart:/dev/tty.usbmodem14201"
```

Mentsd el a fájlt és lépj ki a szerkesztőből.

### 5. PC/SC leállítása (ha szükséges)

Az ACR122U-nál lehet, hogy ütközést okoz a PC/SC démon (`pcscd`). Ha szükséges, állítsd le:

```bash
sudo launchctl stop com.apple.pcscd
```

### 6. Az eszköz kapcsolatának ellenőrzése

Teszteld az NFC olvasót:

```bash
nfc-list
```

Ha az eszköz megfelelően van csatlakoztatva és konfigurálva, a kimenetben megjelenik az NFC olvasó és az esetlegesen észlelt NFC címkék információja.

## ACR122U specifikus konfiguráció

Az ACR122U-nál a következő konfiguráció működhet:

```
device.name = "ACR122U"
device.connstring = "acr122_usb:"
```

Vagy:

```
device.name = "ACR122U"
device.connstring = "acr122_pcsc:"
```

## További információk

- [libnfc GitHub](https://github.com/nfc-tools/libnfc)
- [libnfc Dokumentáció](https://nfc-tools.org/index.php?title=Libnfc)




