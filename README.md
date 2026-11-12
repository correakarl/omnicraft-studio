# 🎮 Omnicraft Studio v1.0 (Next-Gen)

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)](https://tauri.app/)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte)](https://svelte.dev/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind-CSS-38B2AC?logo=tailwind-css)](https://tailwindcss.com/)
[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC_BY--NC--SA_4.0-lightgrey.svg)](LICENSE)
[![Donate PayPal](https://img.shields.io/badge/Donate-PayPal-ff69b4?logo=paypal)](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=thauksephiroth@gmail.com&lc=ES&item_name=Donacion+Omnicraft+Studio&currency_code=USD)

> **Omnicraft Studio** es la suite y gestor multi-instancia de Minecraft de próxima generación. Desarrollado nativamente en **Rust 2021 + Tauri v2** y una interfaz moderna reactiva en **Svelte 5 Runes + TailwindCSS**, ofrece máximo rendimiento, huella de memoria mínima (~40MB RAM), seguridad estricta, soporte multi-proceso simultáneo y aislamiento atómico absoluto.

---

## 💖 Apoyo y Donaciones al Creador

Omnicraft Studio es un software de autoría propia, distribuido de forma **completamente gratuita y de código abierto**, sin funciones de pago ni publicidad. Si este proyecto te resulta útil y deseas apoyar su desarrollo, auditorías de ciberseguridad y mantenimiento continuo:

* 💳 **Donar vía PayPal:** [Hacer una donación voluntaria](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=thauksephiroth@gmail.com&lc=ES&item_name=Donacion+Omnicraft+Studio&currency_code=USD)
* 📧 **Contacto y donaciones directas PayPal:** `thauksephiroth@gmail.com`

---

## 📥 Descarga de Instaladores Precompilados (Linux)

Los paquetes de instalación listos para usar se encuentran en la carpeta [`dist-packages/`](dist-packages/):

| Paquete | Formato | Tamaño | Distribución / Comando de Instalación |
| :--- | :--- | :--- | :--- |
| **[Debian / Ubuntu / Mint / Pop!_OS](dist-packages/omnicraft-studio_1.0.0_amd64.deb)** | `.deb` | ~6.8 MB | `sudo apt install ./omnicraft-studio_1.0.0_amd64.deb` |
| **[Universal Portátil](dist-packages/omnicraft-studio_1.0.0.AppImage)** | `.AppImage` | ~110 MB | `chmod +x *.AppImage && ./omnicraft-studio_1.0.0.AppImage` |
| **[Fedora / openSUSE / RHEL](dist-packages/omnicraft-studio-1.0.0.x86_64.rpm)** | `.rpm` | ~6.8 MB | `sudo dnf install ./omnicraft-studio-1.0.0.x86_64.rpm` |
| **[Arch Linux / Tarball Portátil](dist-packages/omnicraft-studio-v1.0.0-linux-x86_64.tar.gz)** | `.tar.gz` | ~6.6 MB | `tar -xzvf *.tar.gz && ./omnicraft-v2` |

> Para instrucciones paso a paso detalladas, consulta la [Guía en `dist-packages/README.md`](dist-packages/README.md).

---

## 🌟 Características Clave

### 1. 🚀 Motor de Ejecución Directo Nativo en Rust (Runner Zero-Overhead)
* **Sin lanzadores secundarios:** El motor en Rust analiza los descriptores `version.json` de Mojang y modloaders, resuelve dependencias y ejecuta directamente el binario oficial de Java (`KnotClient`, `Main` de Minecraft o bootstrap de Forge/NeoForge).
* **Compatibilidad Total:** Compatible con cuentas **Microsoft Oficiales (MSA)** y cuentas **Offline / Libres**.

### 2. ⚡ Multi-Instancia y Supervisión de Recursos en Tiempo Real
* **Ejecución en Paralelo:** Permite correr múltiples instancias simultáneamente en la misma máquina.
* **Asignación de Usuario Independiente:** Cada instancia posee su propio nombre de jugador configurado (`assigned_username`) para evitar colisiones de nick en servidores o partidas LAN.
* **Restricción de Sesión Única para Cuentas Microsoft:** Si hay una cuenta oficial conectada, se bloquea la duplicación de sesión en paralelo en múltiples instancias para prevenir baneos o errores de token.
* **Monitor Multi-Proceso:** Supervisión en vivo de PID, CPU %, y consumo de memoria RAM por cada instancia activa con botón individual de detención.

### 3. 🎯 Control Predeterminado desde el Navbar
* El selector **`Default:`** en la barra superior permite designar qué instancia se arranca directamente desde el botón principal de juego, mientras que la vista **Instancias** permite lanzar y controlar cualquiera de las demás.

### 4. 🛡️ Ciberseguridad y Blindaje
* **Escáner Anti-Malware Integrado:** Inspección profunda de bytecode `.jar` contra exploits como **Fracturiser**, **BleedingPipe** y conexiones C2.
* **Cuarentena Automática:** Aislamiento de mods maliciosos a `.quarantined`.
* **Protección Log4Shell:** Inyección automática del flag `-Dlog4j2.formatMsgNoLookups=true`.
* **Sanitización de Comandos y Protección Zip-Slip.**

### 5. 🎛️ Modos de Interfaz Dual (Básico 🌱 ↔ Avanzado ⚡)
* **Modo Básico:** Experiencia limpia y minimalista para jugar en 1 clic.
* **Modo Avanzado:** Acceso a presets de Garbage Collector (Aikar, ZGC Generacional, Shenandoah), consola interactiva STDIN de servidores, flags JVM y herramientas avanzadas.

### 6. 🎨 Skin Studio 3D Skindex Pro
* Renderizador interactivo Three.js / skinview3d para modelos Steve y Alex.
* Edición de capas duales (piel base y accesorios exteriores 3D).
* Soporte para inyección en clientes locales y subida a servidores oficiales Mojang.

### 7. ☕ Aprovisionamiento Autónomo de Java (OpenJDK Eclipse Temurin)
* Detección automática en el sistema y descarga en 1-clic de entornos aislados de Java 8, Java 17 y Java 21 LTS en `~/.omnicraft_studio/runtimes/`.

### 8. 🛒 Catálogo Integrado Modrinth & CurseForge
* Búsqueda unificada de Mods, Shaders, Resourcepacks y Modpacks con resolución de dependencias.
* Repositorio de caché compartida para vincular mods entre instancias con **0 MB extra de disco**.

---

## 🛠️ Stack Tecnológico

| Componente | Tecnologías |
| :--- | :--- |
| **Backend / Motor Nativo** | Rust 2021, Tauri v2, SQLite (`rusqlite`), Reqwest, Tokio, sysinfo |
| **Frontend** | Svelte 5 (Runes `$state`, `$derived`, `$effect`), TypeScript, Vite, TailwindCSS |
| **Gráficos 3D** | Three.js, skinview3d |
| **Seguridad** | Hashing criptográfico, Criptografía Fernet para tokens, Bytecode Scanner |

---

## 🚀 Compilación Local desde Código Fuente

### Prerrequisitos
* Rust & Cargo 1.80+: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* Node.js v20+ y `pnpm`: `npm install -g pnpm`
* Dependencias del sistema (Debian/Ubuntu):
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```

### Pasos para compilar
```bash
git clone https://github.com/correakarl/Omnicraft-v2.git
cd Omnicraft-v2

# Instalar dependencias
pnpm install

# Ejecutar en modo desarrollo
pnpm tauri dev

# Compilar binarios y paquetes de distribución
pnpm tauri build
```
Los ejecutables se generan en `src-tauri/target/release/` y los paquetes en `dist-packages/`.

---

## ⚖️ Aviso Legal, Propiedad Intelectual y Licencia

### Propiedad del Proyecto
**Omnicraft Studio** es un software independiente desarrollado por **Karl Correa (`correakarl`)**. 

* **No Afiliación:** Omnicraft Studio **NO** es un producto oficial de Minecraft ni está aprobado por, afiliado ni asociado a **Mojang Studios** ni **Microsoft Corporation**.
* **Marcas Registradas:** *Minecraft* es una marca registrada de Mojang AB / Microsoft. El uso del término en esta documentación es estrictamente descriptivo e informativo conforme a las directrices de directrices comunitarias de marca.
* **Integridad de Código:** Omnicraft Studio no distribuye archivos propietarios protegidos por copyright del juego Minecraft (`minecraft.jar` ni librerías cerradas); estos son descargados directamente por el usuario o provistos por los manifiestos oficiales de Mojang/Modloaders.

### Licencia (CC BY-NC-SA 4.0)
Este software y su código fuente se distribuyen bajo la licencia **Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)**.
* **Atribución (BY):** Reconocimiento de autoría original a Karl Correa.
* **No Comercial (NC):** Prohibido el uso o redistribución con fines de lucro directo o indirecto.
* **Compartir Igual (SA):** Cualquier trabajo derivado debe mantener esta misma licencia.

---
*Omnicraft Studio v1.0 — Potencia, seguridad y libertad multi-instancia.*
