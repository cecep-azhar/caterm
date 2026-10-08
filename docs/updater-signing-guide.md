# Minisign Key Management & Signing Guide (Tauri v2 Updater)

Panduan manajemen kriptografi Minisign untuk rilis aplikasi CATerm.

## 1. Verifikasi Public Key
Public key terpasang di `crates/caterm-app/tauri.conf.json`:
```
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEZBMzQwQzgyQ0ExNjREOApSV1RZWktFc3lFQ2pEMThrQ21uS0p0Vm40cW1NMmxWa3EvUHA0TkpCQVYzc2JtWk5QVndTQkd2cgo=
```

Decode public key:
- Key ID: `FA340C82CA164D8`
- Signature ID: `RWTYZKEsyECjD88kCmnKJtVn4qmM2lVkq/Pp4NJBAV3sbmZNPVwSBGvr`

## 2. Manajemen Private Key Runner CI/CD
Private key Minisign (`TAURI_SIGNING_PRIVATE_KEY`) bersifat rahasia dan TIDAK BOLEH dikomit ke Git repository.

### Penyimpanan Secret:
- **GitHub Actions Secret**: Simpan isi private key (termasuk untrusted comment) di repository secret `TAURI_SIGNING_PRIVATE_KEY`.
- **Passphrase**: Jika private key diproteksi passphrase, set secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

### Format Private Key Minisign (`minisign.key`):
```text
untrusted comment: minisign secret key
RWRTY...[base64 private key]...
```

## 3. Penandatanganan Manual / Lokal
Gunakan tool resmi `minisign`:
```bash
# Menandatangani file biner/installer
minisign -s ~/.minisign/caterm.key -S -m CATerm-x86_64.AppImage -x CATerm-x86_64.AppImage.sig

# Menandatangani via Tauri CLI
export TAURI_SIGNING_PRIVATE_KEY="ISI_PRIVATE_KEY"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
cargo tauri build
```

## 4. Format Manifest `latest.json` (Tauri v2)
Updater membaca manifest dengan skema berikut:
```json
{
  "version": "v2.1.19",
  "notes": "Changelog rilis...",
  "pub_date": "2026-10-08T12:00:00Z",
  "platforms": {
    "linux-x86_64": {
      "signature": "isi signature base64 dari file .sig",
      "url": "https://caterm.fathforce.com/dl/caterm/v2.1.19/CATerm-2.1.19-x86_64.AppImage.tar.gz"
    },
    "windows-x86_64": {
      "signature": "isi signature base64 dari file .sig",
      "url": "https://caterm.fathforce.com/dl/caterm/v2.1.19/CATerm-2.1.19-x64-setup.nsis.zip"
    }
  }
}
```

## 5. Keamanan & Rotasi Key
1. Jika private key bocor:
   - Buat keypair baru dengan `minisign -G`.
   - Update `tauri.conf.json` dengan public key baru.
   - Buat rilis update darurat dan distribusikan sebelum rilis berikutnya.
2. Jangan pernah mencetak isi private key ke runner logs (gunakan secret masking).
