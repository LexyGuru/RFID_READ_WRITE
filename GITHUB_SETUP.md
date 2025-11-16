# GitHub Actions Beállítási Útmutató

Ez az útmutató segít beállítani a GitHub Actions-t az automatikus build folyamathoz.

## 1. GitHub Repository Létrehozása

1. Menj a [GitHub](https://github.com) oldalra
2. Kattints a "New repository" gombra
3. Add meg a repository nevét (pl. `nfc-rust`)
4. Válaszd ki, hogy public vagy private legyen
5. **NE** add hozzá a README-t, .gitignore-t vagy licencet (már megvannak)
6. Kattints a "Create repository" gombra

## 2. Lokális Repository Inicializálása

```bash
# Ha még nincs git repository
git init

# Add hozzá a fájlokat
git add .

# Első commit
git commit -m "Initial commit: NFC Rust Tauri app"

# Add hozzá a remote repository-t (cseréld ki a USERNAME és REPO nevet)
git remote add origin https://github.com/USERNAME/REPO.git

# Push az első commit-ot
git branch -M main
git push -u origin main
```

## 3. GitHub Actions Automatikus Működés

A GitHub Actions automatikusan elindul, amikor:
- Push-t csinálsz a `main` vagy `master` branch-re
- Pull Request-et nyitasz a `main` vagy `master` branch-re
- Manuálisan indítod a "workflow_dispatch" eseménnyel

## 4. Code Signing (Opcionális)

Ha aláírt binárisokat szeretnél (macOS és Windows), be kell állítanod a code signing kulcsokat:

### macOS Code Signing

1. Generáld a kulcsot:
   ```bash
   npm run tauri signer generate -w ~/.tauri/myapp.key
   ```

2. Add hozzá a GitHub Secrets-hez:
   - Menj a repository Settings → Secrets → Actions
   - Add hozzá: `TAURI_PRIVATE_KEY` (a kulcs fájl tartalma)
   - Add hozzá: `TAURI_KEY_PASSWORD` (a jelszó)

### Windows Code Signing

1. Szerez be egy code signing tanúsítványt
2. Add hozzá a GitHub Secrets-hez:
   - `TAURI_SIGNING_PRIVATE_KEY`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

**Megjegyzés:** Code signing nélkül is működik a build, de a binárisok nem lesznek aláírva.

## 5. Build Eredmények Megtekintése

1. Menj a repository Actions fülre
2. Látod az összes build folyamatot
3. Kattints egy build-re a részletek megtekintéséhez

## 6. Release-ek Letöltése

1. Menj a repository Releases fülre
2. Látod az automatikusan generált draft release-eket
3. Publikálhatod őket, vagy letöltheted a binárisokat

## 7. Build Artifacts

Minden build után elérhetőek az artifacts:
1. Menj a build részletekhez
2. Görgess le az "Artifacts" részhez
3. Letöltheted a build fájlokat

## Platform Specifikus Build Eredmények

- **macOS:** `.app` fájl (tar.gz-ben csomagolva)
- **Linux:** `.AppImage` fájl
- **Windows:** `.msi` installer fájl

## Hibaelhárítás

### Build sikertelen

1. Nézd meg a build logokat az Actions fülön
2. Gyakori problémák:
   - Hiányzó függőségek (Linux)
   - Code signing hibák (ha be van állítva)
   - Rust compilation hibák

### Release nem jön létre

- Ellenőrizd, hogy a push a `main` vagy `master` branch-re történt-e
- Ellenőrizd a `GITHUB_TOKEN` permissions-t

### Artifacts nem jelennek meg

- Várj néhány percet, mert a build hosszú időt vehet igénybe
- Ellenőrizd, hogy a build sikeresen befejeződött-e

