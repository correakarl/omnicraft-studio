# 📦 Omnicraft Studio — Descargas Oficiales (Multiplataforma)

En este directorio público se almacenan los instaladores y ejecutables compilados para cada sistema operativo, accesibles directamente por URL pública desde GitHub.

## 🔗 Enlaces de Descarga Directa (`raw.githubusercontent.com`)

| Sistema Operativo | Formato | Ruta en el Repositorio | URL Pública Directa |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu / Mint)** | `.deb` | `downloads/linux/` | `https://github.com/correakarl/omnicraft-studio/raw/main/downloads/linux/` |
| **Linux (Universal Portátil)** | `.AppImage` | `downloads/linux/` | `https://github.com/correakarl/omnicraft-studio/raw/main/downloads/linux/` |
| **Linux (Fedora / RHEL / openSUSE)** | `.rpm` | `downloads/linux/` | `https://github.com/correakarl/omnicraft-studio/raw/main/downloads/linux/` |
| **Windows 10 / 11 (64-bit)** | `.exe` / `.msi` | `downloads/windows/` | `https://github.com/correakarl/omnicraft-studio/raw/main/downloads/windows/` |
| **macOS (Apple Silicon & Intel)** | `.dmg` / `.app.tar.gz` | `downloads/macos/` | `https://github.com/correakarl/omnicraft-studio/raw/main/downloads/macos/` |

---

## 🛠️ Cómo generar y depositar los instaladores desde cada SO

Cuando clones el repositorio en cada sistema operativo, ejecuta:

```bash
pnpm install
pnpm tauri build
```

Y luego copia los instaladores generados en `src-tauri/target/release/bundle/` dentro de su carpeta correspondiente:

- **Linux**: copiar `.deb`, `.AppImage`, `.rpm` a `downloads/linux/`
- **Windows**: copiar `.exe` (`nsis/`) y `.msi` (`msi/`) a `downloads/windows/`
- **macOS**: copiar `.dmg` (`dmg/`) a `downloads/macos/`
