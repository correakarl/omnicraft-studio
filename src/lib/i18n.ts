// Sistema Modular y Extensible de Internacionalización (i18n) para Omnicraft Launcher Studio v1.0
import { invoke } from '@tauri-apps/api/core';

export interface LanguageMeta {
  code: string;
  name: string;
  flag: string;
}

export type TranslationDictionary = Record<string, string>;

// Diccionarios base precargados (Español & English)
const baseTranslations: Record<string, TranslationDictionary> = {
  es: {
    // General
    app_title: 'Omnicraft Launcher Studio',
    loading: 'Cargando...',
    save: 'Guardar',
    cancel: 'Cancelar',
    close: 'Cerrar',
    search: 'Buscar',
    delete: 'Eliminar',
    edit: 'Editar',
    clone: 'Clonar',
    export: 'Exportar',
    import: 'Importar',
    download: 'Descargar',
    downloading: 'Descargando...',
    install: 'Instalar',
    installed: 'Instalado',
    enabled: 'Activado',
    disabled: 'Desactivado',
    status: 'Estado',
    success: 'Éxito',
    error: 'Error',
    warning: 'Advertencia',
    yes: 'Sí',
    no: 'No',

    // Header & Sidebar
    nav_overview: 'General & Lanzador',
    nav_instances: 'Instancias (Perfiles)',
    nav_store: 'Tienda & Recursos',
    nav_mods: 'Mods & Shaders',
    nav_loaders: 'Mod Loaders',
    nav_logs: 'Logs & Consola',
    nav_worlds: 'Mundos & Backups',
    nav_config: 'Ajustes & Java',
    section_instances: 'Instancias',
    section_content: 'Contenido',
    section_advanced: 'Herramientas Avanzadas',
    section_system: 'Configuración',
    sqlite_connected: 'Sistema Conectado',
    mode_basic: '🌱 Modo Básico',
    mode_advanced: '⚡ Modo Avanzado',
    lang_switch: 'Idioma / Language',

    // Overview Tab
    ready_to_play: 'Listo para Jugar',
    server_dedicated: 'Servidor Dedicado',
    launch_game: 'Iniciar Instancia',
    launch_server: 'Iniciar Servidor',
    starting: 'Iniciando...',
    game_running: 'En ejecución',
    stats_memory: 'Memoria Asignada',
    stats_version: 'Versión del Juego',
    stats_loader: 'Cargador de Mods',
    stats_java: 'Entorno Java',
    recent_instances: 'Instancias Recientes',
    no_instances: 'No hay instancias creadas. ¡Crea una para comenzar!',

    // Instances Tab
    create_instance: 'Nueva Instancia',
    filter_all: 'Todas',
    filter_client: 'Clientes',
    filter_server: 'Servidores',
    delete_instance_confirm: '¿Estás seguro de que deseas eliminar esta instancia y todos sus datos?',
    open_folder: 'Abrir Carpeta',
    clone_instance: 'Clonar Instancia',

    // Mod Loaders Tab
    fabric_installer_title: 'Instalador de Fabric Loader',
    fabric_installer_desc: 'Descarga e inyecta el perfil oficial de Fabric directamente desde meta.fabricmc.net sin instaladores externos.',
    install_fabric_btn: 'Instalar Fabric en Instancia',
    paper_installer_title: 'Descargador de Servidores PaperMC',
    paper_installer_desc: 'Descarga el server.jar optimizado y de alto rendimiento de PaperMC v3 al vuelo.',
    download_paper_btn: 'Descargar Paper server.jar',

    // Mods Tab
    mods_title: 'Mods Instalados en la Instancia',
    mods_desc: 'Inspecciona, activa, desactiva o escanea la seguridad de tus mods.',
    scan_all_mods: '🛡️ Escanear Todo con Anti-Malware',
    scan_mod_btn: 'Escanear Bytecode',
    quarantine_btn: 'Cuarentena',
    quarantined_badge: 'EN CUARENTENA',
    no_mods_found: 'No se encontraron mods en esta instancia.',

    // Store Tab
    store_title: 'Explorador Modrinth / CurseForge',
    store_desc: 'Busca e instala mods, shaders y resource packs directamente con resolución de dependencias.',
    search_placeholder: 'Buscar mods, shaders, optimizaciones...',
    filter_mods: 'Mods',
    filter_shaders: 'Shaders',
    filter_resourcepacks: 'Resource Packs',
    filter_modpacks: 'Modpacks',

    // Worlds Tab
    worlds_title: 'Mundos & Copias de Seguridad (Backups)',
    worlds_desc: 'Gestiona los mundos de tu instancia, crea snapshots .zip y restáuralos de forma atómica y segura.',
    create_backup: 'Crear Respaldo (.zip)',
    restore_backup: 'Restaurar Respaldo',
    restore_confirm: '¿Deseas restaurar este respaldo? Se creará una salvaguarda previa de seguridad.',
    no_worlds_found: 'No hay mundos guardados en esta instancia.',

    // Logs & Console Tab
    logs_title: 'Logs y Salida del Proceso en Vivo',
    logs_desc: 'Inspecciona la salida de Minecraft o envía comandos en vivo al servidor dedicado.',
    clear_logs: 'Limpiar Logs',
    server_console_prompt: 'Enviar comando al servidor (ej: /op jugador, /say Hola, /stop)...',
    send_cmd_btn: 'Enviar',

    // Config Tab
    config_title: 'Configuración del Sistema, Credenciales & Entornos Java',
    config_desc: 'Detección nativa de Java, credenciales API y soporte del sistema.',
    api_auth_title: 'Autenticación de Tiendas de Mods (CurseForge / Modrinth)',
    api_key_saved: 'Clave Guardada en SQLite',
    curseforge_key_label: 'CurseForge API Key ($2a$10$... o Personal):',
    modrinth_token_label: 'Modrinth Personal Access Token (Opcional):',
    save_key_btn: 'Guardar',
    javas_detected_title: 'Entornos Java Detectados en el Sistema',
    install_openjdk_label: 'Instalar OpenJDK:',
    java_compatible: 'Compatible',
    language_label: 'Idioma de la Interfaz:',
    language_expandable_hint: 'La arquitectura modular permite incorporar fácilmente nuevos paquetes de idiomas en futuras actualizaciones.',

    // Donations & Credits
    donations_title: 'Omnicraft Launcher Studio v1.0 — Suite Todo-Terreno de Autoría Propia',
    donations_desc: 'Desarrollado de forma libre y gratuita para todos los sistemas operativos, combinando lo mejor de las herramientas más destacadas de la comunidad de Minecraft en una solución unificada, ágil y de alto rendimiento.',
    donate_paypal_btn: 'Apoyar / Donar (PayPal)',
    formal_thanks_title: 'Agradecimiento Formal e Inspiración:',
    formal_thanks_desc: 'Agradecemos profundamente el trabajo e ingenio pionero de HeberonYT (MultiMinecraft 2 Deluxe) por el concepto de aislamiento atómico de carpetas, a Prism Launcher / MultiMC por los formatos abiertos, y a las plataformas Modrinth & CurseForge por sus APIs. Sus aportes nos motivaron a crear esta versión todo-terreno optimizada desde cero.',
    license_badge: 'Licencia CC BY-NC-SA 4.0',
    developed_by: 'Desarrollado con dedicación por Karl Correa',
    contact_btn: 'Contactar',

    // Modals
    account_modal_title: 'Gestor de Cuentas & Perfiles de Jugador',
    offline_account: 'Cuenta Non-Premium (Offline)',
    microsoft_account: 'Cuenta Microsoft (Oficial)',
    add_account: 'Agregar Cuenta',
    device_code_title: 'Vinculación de Cuenta Microsoft',
    device_code_instruction: 'Abre tu navegador en el enlace oficial y escribe este código:',
    copy_code: 'Copiar Código',
    waiting_auth: 'Esperando confirmación en Microsoft...',

    // Skin Studio
    skin_studio_title: 'Skin Studio 3D Skindex Pro',
    skin_base_layer: 'Piel Base',
    skin_outer_layer: '2da Capa Exterior',
    skin_both_layers: 'Ambas Capas',
    skin_preset_steve: 'Steve Clásico',
    skin_preset_alex: 'Alex Delgado',
    save_skin: 'Guardar Skin',
    apply_to_game: 'Aplicar a Instancia'
  },
  en: {
    // General
    app_title: 'Omnicraft Launcher Studio',
    loading: 'Loading...',
    save: 'Save',
    cancel: 'Cancel',
    close: 'Close',
    search: 'Search',
    delete: 'Delete',
    edit: 'Edit',
    clone: 'Clone',
    export: 'Export',
    import: 'Import',
    download: 'Download',
    downloading: 'Downloading...',
    install: 'Install',
    installed: 'Installed',
    enabled: 'Enabled',
    disabled: 'Disabled',
    status: 'Status',
    success: 'Success',
    error: 'Error',
    warning: 'Warning',
    yes: 'Yes',
    no: 'No',

    // Header & Sidebar
    nav_overview: 'Overview & Launcher',
    nav_instances: 'Instances (Profiles)',
    nav_store: 'Store & Resources',
    nav_mods: 'Mods & Shaders',
    nav_loaders: 'Mod Loaders',
    nav_logs: 'Logs & Console',
    nav_worlds: 'Worlds & Backups',
    nav_config: 'Settings & Java',
    section_instances: 'Instances',
    section_content: 'Content',
    section_advanced: 'Advanced Tools',
    section_system: 'System Settings',
    sqlite_connected: 'System Connected',
    mode_basic: '🌱 Basic Mode',
    mode_advanced: '⚡ Advanced Mode',
    lang_switch: 'Language / Idioma',

    // Overview Tab
    ready_to_play: 'Ready to Play',
    server_dedicated: 'Dedicated Server',
    launch_game: 'Launch Instance',
    launch_server: 'Start Server',
    starting: 'Starting...',
    game_running: 'Running',
    stats_memory: 'Assigned Memory',
    stats_version: 'Game Version',
    stats_loader: 'Mod Loader',
    stats_java: 'Java Runtime',
    recent_instances: 'Recent Instances',
    no_instances: 'No instances created yet. Create one to get started!',

    // Instances Tab
    create_instance: 'New Instance',
    filter_all: 'All',
    filter_client: 'Clients',
    filter_server: 'Servers',
    delete_instance_confirm: 'Are you sure you want to delete this instance and all its files?',
    open_folder: 'Open Folder',
    clone_instance: 'Clone Instance',

    // Mod Loaders Tab
    fabric_installer_title: 'Fabric Loader Installer',
    fabric_installer_desc: 'Download and inject official Fabric profiles directly from meta.fabricmc.net without external installers.',
    install_fabric_btn: 'Install Fabric into Instance',
    paper_installer_title: 'PaperMC Server Downloader',
    paper_installer_desc: 'Download optimized, high-performance server.jar builds from PaperMC v3 on the fly.',
    download_paper_btn: 'Download Paper server.jar',

    // Mods Tab
    mods_title: 'Mods Installed in Instance',
    mods_desc: 'Inspect, enable, disable or scan the security of your mods.',
    scan_all_mods: '🛡️ Scan All with Anti-Malware',
    scan_mod_btn: 'Scan Bytecode',
    quarantine_btn: 'Quarantine',
    quarantined_badge: 'QUARANTINED',
    no_mods_found: 'No mods found in this instance.',

    // Store Tab
    store_title: 'Modrinth / CurseForge Explorer',
    store_desc: 'Search and install mods, shaders and resource packs directly with dependency resolution.',
    search_placeholder: 'Search mods, shaders, performance packs...',
    filter_mods: 'Mods',
    filter_shaders: 'Shaders',
    filter_resourcepacks: 'Resource Packs',
    filter_modpacks: 'Modpacks',

    // Worlds Tab
    worlds_title: 'Worlds & Backups',
    worlds_desc: 'Manage your instance saves, create .zip snapshots and restore them atomically and securely.',
    create_backup: 'Create Backup (.zip)',
    restore_backup: 'Restore Backup',
    restore_confirm: 'Do you want to restore this backup? A safety rollback save will be created first.',
    no_worlds_found: 'No saved worlds found in this instance.',

    // Logs & Console Tab
    logs_title: 'Live Process Output & Logs',
    logs_desc: 'Inspect Minecraft stdout/stderr or send live interactive commands to your dedicated server.',
    clear_logs: 'Clear Logs',
    server_console_prompt: 'Send command to server (e.g. /op player, /say Hello, /stop)...',
    send_cmd_btn: 'Send',

    // Config Tab
    config_title: 'System Settings, Credentials & Java Runtimes',
    config_desc: 'Native Java detection, API credentials and platform support.',
    api_auth_title: 'Mod Store Authentication (CurseForge / Modrinth)',
    api_key_saved: 'Key Saved in SQLite',
    curseforge_key_label: 'CurseForge API Key ($2a$10$... or Personal):',
    modrinth_token_label: 'Modrinth Personal Access Token (Optional):',
    save_key_btn: 'Save',
    javas_detected_title: 'Detected Java Runtimes in System',
    install_openjdk_label: 'Install OpenJDK:',
    java_compatible: 'Compatible',
    language_label: 'Interface Language:',
    language_expandable_hint: 'The modular architecture allows adding new language packs seamlessly in future updates.',

    // Donations & Credits
    donations_title: 'Omnicraft Launcher Studio v1.0 — All-in-One Suite of Original Authorship',
    donations_desc: 'Developed freely and openly for all operating systems, combining the best features from standout Minecraft tools into a unified, agile, high-performance solution.',
    donate_paypal_btn: 'Support / Donate (PayPal)',
    formal_thanks_title: 'Formal Acknowledgements & Inspiration:',
    formal_thanks_desc: 'We deeply thank the pioneering work of HeberonYT (MultiMinecraft 2 Deluxe) for the atomic .minecraft directory isolation concept, Prism Launcher / MultiMC for open formats, and Modrinth & CurseForge for their public APIs. Their work inspired us to build this all-terrain, hardened version from scratch.',
    license_badge: 'License CC BY-NC-SA 4.0',
    developed_by: 'Crafted with dedication by Karl Correa',
    contact_btn: 'Contact',

    // Modals
    account_modal_title: 'Account & Player Profile Manager',
    offline_account: 'Non-Premium (Offline) Account',
    microsoft_account: 'Microsoft (Official) Account',
    add_account: 'Add Account',
    device_code_title: 'Microsoft Account Linking',
    device_code_instruction: 'Open your web browser at the official link and enter this code:',
    copy_code: 'Copy Code',
    waiting_auth: 'Waiting for Microsoft confirmation...',

    // Skin Studio
    skin_studio_title: 'Skin Studio 3D Skindex Pro',
    skin_base_layer: 'Base Skin',
    skin_outer_layer: 'Outer Layer (Overlays)',
    skin_both_layers: 'Both Layers',
    skin_preset_steve: 'Classic Steve',
    skin_preset_alex: 'Slim Alex',
    save_skin: 'Save Skin',
    apply_to_game: 'Apply to Instance'
  }
};

class I18nService {
  currentLang: string = 'es';
  private languages: LanguageMeta[] = [
    { code: 'es', name: 'Español', flag: '🇪🇸' },
    { code: 'en', name: 'English', flag: '🇺🇸' }
  ];
  private registries: Record<string, TranslationDictionary> = { ...baseTranslations };
  private listeners: Array<(lang: string) => void> = [];

  /**
   * Inicializa el idioma recuperado desde SQLite o predeterminado
   */
  async init(): Promise<string> {
    try {
      const savedLang = await invoke<string>('get_system_language');
      if (savedLang && (this.registries[savedLang] || savedLang === 'es' || savedLang === 'en')) {
        this.currentLang = savedLang;
      }
    } catch {
      this.currentLang = 'es';
    }
    this.notify();
    return this.currentLang;
  }

  /**
   * Lista todos los idiomas disponibles registrados
   */
  getAvailableLanguages(): LanguageMeta[] {
    return [...this.languages];
  }

  /**
   * Permite registrar dinámicamente un nuevo paquete de idioma en futuras versiones o mods
   */
  registerLanguage(meta: LanguageMeta, dictionary: TranslationDictionary) {
    if (!this.languages.some(l => l.code === meta.code)) {
      this.languages.push(meta);
    }
    this.registries[meta.code] = {
      ...(this.registries[meta.code] || {}),
      ...dictionary
    };
    this.notify();
  }

  /**
   * Cambia el idioma activo y lo persiste en la base de datos SQLite
   */
  async setLanguage(langCode: string): Promise<void> {
    this.currentLang = langCode;
    try {
      await invoke('set_system_language', { lang: langCode });
    } catch (err) {
      console.error('Error al guardar idioma en SQLite:', err);
    }
    this.notify();
  }

  /**
   * Obtiene la traducción para una clave dada, con fallback seguro a español
   */
  t(key: string, fallback?: string): string {
    const dict = this.registries[this.currentLang] || this.registries['es'] || {};
    if (dict[key]) return dict[key];

    // Fallback al español
    const fallbackDict = this.registries['es'] || {};
    return fallbackDict[key] || fallback || key;
  }

  /**
   * Suscribe componentes para re-renderizarse ante cambios de idioma
   */
  subscribe(listener: (lang: string) => void): () => void {
    this.listeners.push(listener);
    listener(this.currentLang);
    return () => {
      this.listeners = this.listeners.filter(l => l !== listener);
    };
  }

  private notify() {
    for (const listener of this.listeners) {
      listener(this.currentLang);
    }
  }
}

export const i18n = new I18nService();
export const t = (key: string, fallback?: string): string => { try { return i18n.t(key, fallback); } catch { return fallback || key; } };
