# 📦 Binarios e Instaladores de Omnicraft Studio v1.0

En esta carpeta se alojan los paquetes precompilados listos para su instalación y ejecución directa en sistemas operativos GNU/Linux.

---

## 🚀 Guía de Instalación Rápida

### 1. 🐧 Debian / Ubuntu / Linux Mint / Pop!_OS (`.deb`)
Descarga o ubica el archivo `.deb` y ejecuta en terminal:
```bash
sudo apt install ./omnicraft-studio_1.0.0_amd64.deb
```
O con `dpkg`:
```bash
sudo dpkg -i omnicraft-studio_1.0.0_amd64.deb
sudo apt-get install -f # Si faltan librerías
```
*Se integrará automáticamente en el menú de aplicaciones de tu escritorio.*

---

### 2. ⚡ Paquete Portátil Universal (`.AppImage`)
Funciona en cualquier distribución de Linux (Arch, Fedora, Debian, Manjaro, Ubuntu, SteamOS) sin necesidad de instalar:
```bash
chmod +x omnicraft-studio_1.0.0.AppImage
./omnicraft-studio_1.0.0.AppImage
```

---

### 3. 🎩 Fedora / Red Hat / openSUSE (`.rpm`)
Para sistemas basados en RPM:
```bash
sudo dnf install ./omnicraft-studio-1.0.0.x86_64.rpm
```
O con `zypper` en openSUSE:
```bash
sudo zypper install ./omnicraft-studio-1.0.0.x86_64.rpm
```

---

### 4. 🗜️ Arch Linux / Tarball Portátil (`.tar.gz`)
Si prefieres el binario sin empaquetar o utilizas Arch Linux / Void:
```bash
tar -xzvf omnicraft-studio-v1.0.0-linux-x86_64.tar.gz
./omnicraft-v2
```

---

## 🔒 Verificación de Integridad y Seguridad
Todos los binarios fueron compilados de forma reproducible con Rust 1.80+ y Tauri v2 en entorno aislado, con sanitización de argumentos JVM, mitigación de vulnerabilidades Log4j / Zip-Slip y sin telemetría ni analíticas invasivas.
