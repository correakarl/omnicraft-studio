# 📦 Omnicraft Studio — Descargas Oficiales (Multiplataforma)

En este directorio público se almacenan los instaladores y ejecutables compilados para cada sistema operativo, accesibles directamente por URL pública desde GitHub.

## 🔗 Enlaces de Descarga Directa (`raw.githubusercontent.com`)

| Sistema Operativo | Formato | Archivo | Tamaño | URL Pública Directa |
| :--- | :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu / Mint)** | `.deb` | `Omnicraft.Studio_1.0.0_amd64.deb` | 6.7 MB | [Descargar `.deb`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/linux/Omnicraft.Studio_1.0.0_amd64.deb) |
| **Linux (Universal Portátil)** | `.AppImage` | `Omnicraft.Studio_1.0.0_amd64.AppImage` | 83.1 MB | [Descargar `.AppImage`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/linux/Omnicraft.Studio_1.0.0_amd64.AppImage) |
| **Linux (Fedora / RHEL / openSUSE)** | `.rpm` | `Omnicraft.Studio-1.0.0-1.x86_64.rpm` | 6.7 MB | [Descargar `.rpm`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/linux/Omnicraft.Studio-1.0.0-1.x86_64.rpm) |
| **Windows 10 / 11 (Instalador NSIS)** | `.exe` | `Omnicraft.Studio_1.0.0_x64-setup.exe` | 4.4 MB | [Descargar `.exe`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/windows/Omnicraft.Studio_1.0.0_x64-setup.exe) |
| **Windows 10 / 11 (Paquete MSI)** | `.msi` | `Omnicraft.Studio_1.0.0_x64_en-US.msi` | 6.2 MB | [Descargar `.msi`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/windows/Omnicraft.Studio_1.0.0_x64_en-US.msi) |
| **macOS (Instalador DMG)** | `.dmg` | `Omnicraft.Studio_1.0.0_aarch64.dmg` | 7.1 MB | [Descargar `.dmg`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/macos/Omnicraft.Studio_1.0.0_aarch64.dmg) |
| **macOS (Bundle `.app.tar.gz`)** | `.app.tar.gz` | `Omnicraft.Studio_aarch64.app.tar.gz` | 6.5 MB | [Descargar `.app.tar.gz`](https://github.com/correakarl/omnicraft-studio/raw/main/downloads/macos/Omnicraft.Studio_aarch64.app.tar.gz) |

---

## 🛠️ Cómo generar y depositar los instaladores manualmente desde cada SO

Cuando clones el repositorio en cada sistema operativo, ejecuta:

```bash
pnpm install
pnpm tauri build
```

Y luego copia los instaladores generados en `src-tauri/target/release/bundle/` dentro de su carpeta correspondiente:

- **Linux**: copiar `.deb`, `.AppImage`, `.rpm` a `downloads/linux/`
- **Windows**: copiar `.exe` (`nsis/`) y `.msi` (`msi/`) a `downloads/windows/`
- **macOS**: copiar `.dmg` (`dmg/`) a `downloads/macos/`
