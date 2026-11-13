<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Play,
    Server,
    Download,
    Layers,
    Package,
    Settings,
    Shield,
    Activity,
    CheckCircle2,
    XCircle,
    RotateCw,
    Search,
    ExternalLink,
    Coffee,
    Cpu,
    HardDrive,
    Trash2,
    AlertTriangle,
    Check,
    Wrench,
    FolderOpen,
    Edit3,
    Plus,
    User,
    Sparkles,
    Filter,
    Globe,
    Terminal,
    Share2,
    Archive,
    History,
    ToggleLeft,
    ToggleRight,
    Heart,
    Mail,
    Languages,
    CheckCircle,
    Info,
    ListFilter,
    X
  } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type {
    InstanceModel,
    AccountModel,
    JavaRuntime,
    RecommendationModel,
    ModItemModel,
    ModSearchResult,
    GameVersionEntry,
    LocalSkinModel,
    WorldSaveModel,
    InstanceLogOutput,
    ContentItemModel,
    AppUpdateModel,
    ScanResultModel,
    DownloadTaskModel,
    CompatibilityReportModel,
    SharedCacheItemModel,
    InstanceResourceUsage
  } from './lib/types';
  import { i18n, t } from './lib/i18n';
  import InstanceModal from './lib/components/InstanceModal.svelte';
  import AccountModal from './lib/components/AccountModal.svelte';
  import SkinEditorModal from './lib/components/SkinEditorModal.svelte';
  import { SkinService } from './lib/skinService';

  // Estados reactivos (Svelte 5 runes)
  let currentLang = $state<string>('es');
  let downloadTasks = $state<DownloadTaskModel[]>([]);
  let showDownloadsDrawer = $state(false);
  let compatibilityReport = $state<CompatibilityReportModel | null>(null);
  let isCheckingCompatibility = $state(false);
  let availableLanguages = $state(i18n.getAvailableLanguages());
  let uiMode = $state<'basic' | 'advanced'>('basic');
  let appUpdate = $state<AppUpdateModel | null>(null);
  let isCheckingUpdate = $state(false);
  let isUpdateModalOpen = $state(false);
  let scanModResults = $state<Record<string, ScanResultModel>>({});
  let isScanningMod = $state<string | null>(null);

  let activeTab = $state<'overview' | 'instances' | 'loaders' | 'mods' | 'store' | 'worlds' | 'logs' | 'config'>('overview');
  let instances = $state<InstanceModel[]>([]);
  let accounts = $state<AccountModel[]>([]);
  let javas = $state<JavaRuntime[]>([]);
  let officialRecs = $state<RecommendationModel[]>([]);
  let instanceMods = $state<ModItemModel[]>([]);
  let availableIcons = $state<string[]>([]);
  let launchers = $state<string[]>([]);
  let allGameVersions = $state<GameVersionEntry[]>([]);

  // Herramientas de Creador (Mundos, Backups, Logs y Exportación)
  let instanceWorlds = $state<WorldSaveModel[]>([]);
  let instanceLogs = $state<InstanceLogOutput>({ lines: [], total_lines: 0, last_updated: '' });
  let isBackingUp = $state(false);
  let isExporting = $state(false);
  let isAutoRefreshLogs = $state(false);
  let logRefreshTimer: any = null;

  let selectedInstanceId = $state<string>('');
  let isLaunching = $state(false);
  let isInstanceRunning = $state(false);
  let instanceResourceUsage = $state<InstanceResourceUsage | null>(null);
  let allRunningUsages = $state<InstanceResourceUsage[]>([]);
  let runningMonitorTimer: any = null;
  let coreStatus = $state('Iniciando servicios del sistema...');

  // Modales
  let isInstanceModalOpen = $state(false);
  let instanceModalMode = $state<'create' | 'edit'>('create');
  let isAccountModalOpen = $state(false);
  let isSkinEditorModalOpen = $state(false);
  let localSkins = $state<LocalSkinModel[]>([]);

  // Búsqueda en tienda unificada (Modrinth + CurseForge)
  let searchQuery = $state('');
  let searchResults = $state<ModSearchResult[]>([]);
  let isSearching = $state(false);
  let storeSource = $state<'all' | 'modrinth' | 'curseforge'>('all');
  let storeType = $state<'mod' | 'shader' | 'resourcepack' | 'modpack'>('mod');
  let storeLoaderFilter = $state<string>('Auto');
  let storeMcFilter = $state<string>('Auto');
  let storeSort = $state<'relevance' | 'downloads' | 'newest'>('downloads');

  // Configuración de API Keys para tiendas (CurseForge / Modrinth)
  let curseforgeApiKey = $state('');
  let modrinthToken = $state('');
  let isApiKeySaved = $state(false);

  // Selección múltiple de mods en tabla
  let selectedModFiles = $state<Set<string>>(new Set());

  let currentInstance = $derived<InstanceModel>(
    instances.find(i => i.id === selectedInstanceId) || instances[0] || {
      id: 'Default',
      name: 'Omnicraft Client',
      folder: 'Default',
      icon: 'art-Crafting_Table.png',
      instance_type: 'client' as const,
      server_port: 25565,
      mc_version: '1.21.4',
      loader_type: 'Fabric',
      executable_type: 'jar_bundle',
      launcher_jar: 'HMCL.jar',
      java_selection_mode: 'auto',
      window_launch_behavior: 'minimize',
      ram_min_mb: 1024,
      ram_max_mb: 4096,
      jvm_preset: 'Aikar Flags',
      width: 1280,
      height: 720,
    }
  );

  let activeAccount = $derived<AccountModel>(
    accounts.find(a => a.active) || accounts[0] || {
      id: 'default',
      username: 'Steve',
      account_type: 'offline' as const,
      uuid: null,
      active: true
    }
  );

  async function loadDataFromRust() {
    try {
      // Inicializar idioma i18n
      currentLang = await i18n.init();
      coreStatus = await invoke<string>('status');
      instances = await invoke<InstanceModel[]>('get_instances');
      if (instances.length > 0 && !selectedInstanceId) {
        selectedInstanceId = instances[0].id;
      }
      accounts = await invoke<AccountModel[]>('get_accounts');
      javas = await invoke<JavaRuntime[]>('detect_javas');
      availableIcons = await invoke<string[]>('list_available_icons');
      launchers = await invoke<string[]>('list_launchers');
      officialRecs = await invoke<RecommendationModel[]>('get_official_recs', { category: 'modpack' });
      allGameVersions = await invoke<GameVersionEntry[]>('get_all_game_versions');
      localSkins = await invoke<LocalSkinModel[]>('get_local_skins');

      // Cargar modo de interfaz global (Básico / Avanzado)
      try {
        const savedMode = await invoke<string>('get_system_ui_mode');
        if (savedMode === 'basic' || savedMode === 'advanced') {
          uiMode = savedMode;
        }
      } catch (_) {}

      // Comprobar actualizaciones de Omnicraft en segundo plano
      checkAppUpdates();

      // Cargar llaves de API guardadas si existen
      const savedCfKey = await invoke<string | null>('get_api_key', { service: 'curseforge' });
      if (savedCfKey) curseforgeApiKey = savedCfKey;

      if (selectedInstanceId) {
        await loadModsForInstance(selectedInstanceId);
        const inst = instances.find(i => i.id === selectedInstanceId);
        if (inst) {
          await loadContentForInstance(inst.folder, activeContentCategory);
        }
        await runCompatibilityCheck();
      }
      doSearchStore();

      // Escuchar eventos de cola de descargas en vivo desde Rust
      try {
        await listen<{ id: string; title: string; source: string; status: string; progress: number; completed: boolean }>('download-event', (event) => {
          const payload = event.payload;
          const idx = downloadTasks.findIndex(t => t.id === payload.id);
          if (idx >= 0) {
            downloadTasks[idx] = payload;
          } else {
            downloadTasks = [payload, ...downloadTasks];
          }
          showDownloadsDrawer = true;
        });
      } catch (evtErr) {
        console.warn('Error registrando listener de descargas:', evtErr);
      }

      // Iniciar ciclo de monitoreo continuo de ejecución de instancia y métricas
      checkRunningState();
      if (!runningMonitorTimer) {
        runningMonitorTimer = setInterval(() => {
          checkRunningState();
        }, 1500);
      }
    } catch (err) {
      console.warn('IPC Rust:', err);
    }
  }

  async function saveApiKey(service: string, keyVal: string) {
    try {
      await invoke('set_api_key', { service, keyValue: keyVal.trim() });
      isApiKeySaved = true;
      setTimeout(() => isApiKeySaved = false, 2500);
    } catch (e) {
      alert(`Error al guardar clave: ${e}`);
    }
  }

  let contentItems = $state<ContentItemModel[]>([]);
  let activeContentCategory = $state<'mods' | 'shaderpacks' | 'resourcepacks'>('mods');
  let contentSearchFilter = $state('');

  // Conjunto reactivo de nombres y slugs instalados en la instancia actual
  let installedInCurrentInstance = $derived.by(() => {
    const set = new Set<string>();
    // Mods registrados en base de datos
    for (const m of instanceMods) {
      if (m.mod_id) set.add(m.mod_id.toLowerCase().trim());
      if (m.name) set.add(m.name.toLowerCase().trim());
      if (m.filename) {
        set.add(m.filename.toLowerCase().trim());
        set.add(m.filename.replace(/\.jar(\.disabled)?$/i, '').toLowerCase().trim());
      }
    }
    // Elementos en el sistema de archivos de la instancia (mods, shaderpacks, resourcepacks)
    for (const c of contentItems) {
      if (c.name) set.add(c.name.toLowerCase().trim());
      if (c.filename) {
        set.add(c.filename.toLowerCase().trim());
        set.add(c.filename.replace(/\.(jar|zip|disabled)+$/i, '').toLowerCase().trim());
      }
    }
    return set;
  });

  function isItemInstalled(item: ModSearchResult): boolean {
    if (!item) return false;
    const s = installedInCurrentInstance;
    const titleNorm = (item.title || '').toLowerCase().trim();
    const slugNorm = (item.slug || '').toLowerCase().trim();
    const idNorm = (item.id || '').toLowerCase().trim();

    if (s.has(titleNorm) || s.has(slugNorm) || s.has(idNorm)) return true;

    // Comparación parcial de coincidencia en archivos para nombres similares
    for (const entry of s) {
      if (titleNorm && (entry.includes(titleNorm) || titleNorm.includes(entry))) return true;
      if (slugNorm && (entry.includes(slugNorm) || slugNorm.includes(entry))) return true;
    }
    return false;
  }

  let filteredContentItems = $derived(
    contentItems.filter(item => item.name.toLowerCase().includes(contentSearchFilter.toLowerCase()))
  );

  let sharedCacheItems = $state<SharedCacheItemModel[]>([]);
  let showSharedCacheModal = $state(false);
  let isLinkingShared = $state(false);

  async function openSharedCacheModal() {
    try {
      sharedCacheItems = await invoke<SharedCacheItemModel[]>('list_shared_cache_items', {
        folderName: currentInstance.folder,
        category: activeContentCategory
      });
      showSharedCacheModal = true;
    } catch (e) {
      console.warn('Error listando caché compartida:', e);
    }
  }

  async function handleLinkSharedItem(filename: string) {
    if (isLinkingShared) return;
    isLinkingShared = true;
    try {
      await invoke('link_shared_item_to_instance', {
        folderName: currentInstance.folder,
        instanceId: currentInstance.id,
        category: activeContentCategory,
        filename
      });
      await loadContentForInstance(currentInstance.folder, activeContentCategory);
      await loadModsForInstance(currentInstance.id);
      await runCompatibilityCheck();
      // Refrescar lista modal
      sharedCacheItems = await invoke<SharedCacheItemModel[]>('list_shared_cache_items', {
        folderName: currentInstance.folder,
        category: activeContentCategory
      });
    } catch (e) {
      alert(`Error al vincular elemento: ${e}`);
    } finally {
      isLinkingShared = false;
    }
  }

  async function loadContentForInstance(folder: string, category: 'mods' | 'shaderpacks' | 'resourcepacks') {
    try {
      contentItems = await invoke<ContentItemModel[]>('list_instance_content', {
        folderName: folder,
        category: category
      });
    } catch (e) {
      console.warn('Error cargando contenido:', e);
    }
  }

  async function handleToggleContent(item: ContentItemModel) {
    try {
      const newName = await invoke<string>('toggle_instance_content', {
        folderName: currentInstance.folder,
        category: item.category,
        filename: item.filename,
        enable: !item.enabled,
        instanceId: currentInstance.id
      });
      await loadContentForInstance(currentInstance.folder, activeContentCategory);
      await loadModsForInstance(currentInstance.id);
    } catch (e) {
      alert(`Error conmutando estado: ${e}`);
    }
  }

  async function handleDeleteContent(item: ContentItemModel) {
    if (!confirm(`¿Eliminar permanentemente '${item.name}'?`)) return;
    try {
      await invoke('delete_instance_content', {
        folderName: currentInstance.folder,
        category: item.category,
        filename: item.filename,
        instanceId: currentInstance.id
      });
      await loadContentForInstance(currentInstance.folder, activeContentCategory);
      await loadModsForInstance(currentInstance.id);
    } catch (e) {
      alert(`Error eliminando archivo: ${e}`);
    }
  }

  async function loadModsForInstance(instId: string) {
    try {
      instanceMods = await invoke<ModItemModel[]>('get_instance_mods', { instanceId: instId });
      selectedModFiles.clear();
      await loadContentForInstance(currentInstance.folder, activeContentCategory);
    } catch (e) {
      console.error(e);
    }
  }

  async function handleSaveLocalSkin(skin: LocalSkinModel) {
    try {
      await invoke('save_local_skin', {
        id: skin.id,
        name: skin.name,
        modelType: skin.model_type,
        base64Png: skin.png_data_base64
      });
      localSkins = await invoke<LocalSkinModel[]>('get_local_skins');
      alert(`✅ Skin '${skin.name}' guardada correctamente en la base de datos local SQLite.`);
    } catch (e) {
      alert(`Error al guardar skin: ${e}`);
    }
  }

  async function handleInjectOfflineSkin(base64: string) {
    try {
      await invoke('inject_offline_skin_to_instance', {
        folderName: currentInstance.folder,
        username: activeAccount.username,
        base64Png: base64
      });
      alert(`✅ Skin inyectada para el jugador '${activeAccount.username}' en la instancia '${currentInstance.name}' (CustomSkinLoader/LocalSkin). Se verá en tu juego offline.`);
    } catch (e) {
      alert(`Error al inyectar skin offline: ${e}`);
    }
  }

  async function handleUploadMojang(base64: string, modelType: string) {
    if (activeAccount.account_type !== 'microsoft') {
      alert('Solo las cuentas oficiales autenticadas con Microsoft pueden actualizar la skin en los servidores de Mojang.');
      return;
    }
    const token = (activeAccount as any).access_token || '';
    try {
      await invoke('upload_skin_to_mojang', {
        username: activeAccount.username,
        bearerToken: token || null,
        base64Png: base64,
        modelType: modelType
      });
      alert('✅ Skin subida con éxito a los servidores oficiales de Mojang / Minecraft.');
    } catch (e) {
      alert(`Error al subir skin a Mojang: ${e}`);
    }
  }

  async function handleDeleteLocalSkin(id: string) {
    if (!confirm('¿Deseas eliminar esta skin de la biblioteca local?')) return;
    try {
      await invoke('delete_local_skin', { id });
      localSkins = await invoke<LocalSkinModel[]>('get_local_skins');
    } catch (e) {
      alert(`Error al eliminar skin: ${e}`);
    }
  }

  async function handleRefreshLocalSkins() {
    try {
      localSkins = await invoke<LocalSkinModel[]>('get_local_skins');
    } catch (e) {
      console.error('Error al recargar skins locales:', e);
    }
  }

  async function openExternalUrl(url: string) {
    try {
      await invoke('open_url', { url });
    } catch (err) {
      console.error('Error abriendo enlace externo:', err);
      window.open(url, '_blank');
    }
  }

  let isInstallingLoader = $state(false);
  let isDownloadingServer = $state(false);

  async function handleInstallFabric() {
    isInstallingLoader = true;
    try {
      const res = await invoke<string>('install_fabric_loader_cmd', {
        folderName: currentInstance.folder,
        mcVersion: currentInstance.mc_version,
      });
      alert(`✅ ${res}`);
    } catch (e) {
      alert(`Error instalando Fabric: ${e}`);
    } finally {
      isInstallingLoader = false;
    }
  }

  async function handleDownloadPaperServer() {
    isDownloadingServer = true;
    try {
      const res = await invoke<string>('download_paper_server_cmd', {
        folderName: currentInstance.folder,
        mcVersion: currentInstance.mc_version,
      });
      alert(`✅ ${res}`);
    } catch (e) {
      alert(`Error descargando PaperMC: ${e}`);
    } finally {
      isDownloadingServer = false;
    }
  }

  let isDownloadingJre = $state<number | null>(null);

  async function handleDownloadTemurinJre(ver: number) {
    isDownloadingJre = ver;
    try {
      const res = await invoke<string>('install_temurin_jre_cmd', { majorVersion: ver });
      alert(`✅ ${res}`);
      javas = await invoke<JavaRuntime[]>('detect_javas');
    } catch (e) {
      alert(`Error descargando Java ${ver}: ${e}`);
    } finally {
      isDownloadingJre = null;
    }
  }

  let serverInputCmd = $state('');

  async function handleSendServerCommand() {
    if (!serverInputCmd.trim()) return;
    try {
      await invoke('send_server_cmd', {
        instanceId: currentInstance.id,
        command: serverInputCmd.trim(),
      });
      serverInputCmd = '';
      setTimeout(() => loadLogsForInstance(currentInstance.folder), 300);
    } catch (e) {
      alert(`Error al enviar comando: ${e}`);
    }
  }

  async function toggleUiMode() {
    const nextMode = uiMode === 'basic' ? 'advanced' : 'basic';
    uiMode = nextMode;
    try {
      await invoke('set_system_ui_mode', { mode: nextMode });
    } catch (e) {
      console.warn('Error guardando modo UI:', e);
    }
  }

  function handleLanguageChange(newLang: string) {
    currentLang = newLang;
    i18n.setLanguage(newLang);
  }

  async function checkAppUpdates() {
    isCheckingUpdate = true;
    try {
      appUpdate = await invoke<AppUpdateModel>('check_omnicraft_updates');
    } catch (e) {
      console.warn('Error verificando actualizaciones:', e);
    } finally {
      isCheckingUpdate = false;
    }
  }

  async function handleScanMod(filename: string) {
    isScanningMod = filename;
    try {
      const res = await invoke<ScanResultModel>('scan_mod_file', {
        folderName: currentInstance.folder,
        filename,
      });
      scanModResults[filename] = res;
      if (!res.is_clean) {
        if (confirm(`⚠️ ${res.description}\n\n¿Deseas aislar y poner en CUARENTENA este archivo de inmediato para proteger tu sistema?`)) {
          await invoke('quarantine_mod_file', {
            folderName: currentInstance.folder,
            filename,
          });
          await loadModsForInstance(currentInstance.id);
          alert('🛡️ Mod movido a la carpeta .quarantine de forma segura.');
        }
      }
    } catch (e) {
      alert(`Error durante el análisis del mod: ${e}`);
    } finally {
      isScanningMod = null;
    }
  }

  async function handleRestoreWorld(backupFilename: string) {
    if (!confirm(`¿Restaurar el respaldo '${backupFilename}'?\nSe recuperará el estado guardado y se creará una copia de protección temporal antes de proceder.`)) return;
    try {
      const msg = await invoke<string>('restore_instance_world', {
        folderName: currentInstance.folder,
        backupFilename,
      });
      await loadWorldsForInstance(currentInstance.folder);
      alert(`✅ ${msg}`);
    } catch (e) {
      alert(`Error al restaurar: ${e}`);
    }
  }

  async function loadWorldsForInstance(folder: string) {
    try {
      instanceWorlds = await invoke<WorldSaveModel[]>('list_instance_worlds', { folderName: folder });
    } catch (e) {
      console.warn('Error listando mundos:', e);
    }
  }

  async function handleBackupWorld(worldName: string) {
    if (isBackingUp) return;
    isBackingUp = true;
    try {
      const filename = await invoke<string>('backup_instance_world', {
        folderName: currentInstance.folder,
        worldName: worldName
      });
      alert(`✅ ¡Copia de seguridad creada con éxito!\nGuardada en .minecraft/backups/${filename}`);
    } catch (e) {
      alert(`Error creando backup: ${e}`);
    } finally {
      isBackingUp = false;
    }
  }

  async function loadLogsForInstance(folder: string) {
    try {
      instanceLogs = await invoke<InstanceLogOutput>('get_instance_logs', { folderName: folder, maxLines: 500 });
    } catch (e) {
      console.warn('Error cargando logs:', e);
    }
  }

  async function handleExportModpack(includeWorlds: boolean) {
    if (isExporting) return;
    isExporting = true;
    try {
      const path = await invoke<string>('export_modpack', {
        folderName: currentInstance.folder,
        exportName: `${currentInstance.name}_Pack`,
        includeWorlds: includeWorlds
      });
      alert(`✅ ¡Paquete exportado con éxito!\nArchivo ZIP listo para compartir con tus seguidores:\n${path}`);
    } catch (e) {
      alert(`Error exportando modpack: ${e}`);
    } finally {
      isExporting = false;
    }
  }

  $effect(() => {
    if (activeTab === 'worlds' && currentInstance.folder) {
      loadWorldsForInstance(currentInstance.folder);
    } else if (activeTab === 'logs' && currentInstance.folder) {
      loadLogsForInstance(currentInstance.folder);
      if (isAutoRefreshLogs && !logRefreshTimer) {
        logRefreshTimer = setInterval(() => loadLogsForInstance(currentInstance.folder), 2500);
      }
    } else {
      if (logRefreshTimer) {
        clearInterval(logRefreshTimer);
        logRefreshTimer = null;
      }
    }
  });

  async function doSearchStore() {
    isSearching = true;
    try {
      const mcVer = storeMcFilter === 'Auto' ? currentInstance.mc_version : (storeMcFilter === 'Todas' ? '' : storeMcFilter);
      const ldr = storeLoaderFilter === 'Auto' ? currentInstance.loader_type.toLowerCase() : (storeLoaderFilter === 'Todos' ? '' : storeLoaderFilter.toLowerCase());

      const res = await invoke<ModSearchResult[]>('search_store_unified', {
        source: storeSource,
        query: searchQuery,
        projectType: storeType,
        mcVersion: mcVer,
        loader: ldr,
        limit: 24,
      });

      searchResults = res;
    } catch (err) {
      console.error('Error buscando en la tienda:', err);
    } finally {
      isSearching = false;
    }
  }

  async function runCompatibilityCheck() {
    if (!currentInstance || !currentInstance.folder) return;
    isCheckingCompatibility = true;
    try {
      compatibilityReport = await invoke<CompatibilityReportModel>('check_instance_compatibility', {
        folderName: currentInstance.folder,
        mcVersion: currentInstance.mc_version,
        loader: currentInstance.loader_type
      });
    } catch (e) {
      console.warn('Error verificando compatibilidad:', e);
    } finally {
      isCheckingCompatibility = false;
    }
  }

  let downloadingItems = $state<Set<string>>(new Set());

  async function handleInstallMod(item: ModSearchResult) {
    if (downloadingItems.has(item.id)) return;
    downloadingItems.add(item.id);
    showDownloadsDrawer = true;
    try {
      const filename = await invoke<string>('install_mod_to_instance', {
        source: item.source || 'Modrinth',
        projectId: item.id,
        instanceId: currentInstance.id,
        folderName: currentInstance.folder,
        mcVersion: currentInstance.mc_version,
        loader: currentInstance.loader_type,
        projectType: item.project_type,
        modName: item.title
      });
      await loadModsForInstance(currentInstance.id);
      await loadContentForInstance(currentInstance.folder, activeContentCategory);
      await runCompatibilityCheck();
    } catch (err) {
      alert(`Error al descargar ${item.title}: ${err}`);
    } finally {
      downloadingItems.delete(item.id);
    }
  }

  async function handleInstanceChange(newInstanceId: string) {
    selectedInstanceId = newInstanceId;
    const targetInst = instances.find(i => i.id === newInstanceId);
    if (targetInst) {
      await loadModsForInstance(targetInst.id);
      await loadContentForInstance(targetInst.folder, activeContentCategory);
      await runCompatibilityCheck();
      if (activeTab === 'store') {
        doSearchStore();
      }
    }
  }

  async function checkRunningState() {
    try {
      allRunningUsages = await invoke<InstanceResourceUsage[]>('get_all_instances_resource_usage');
      const found = allRunningUsages.find(p => p.instance_id === currentInstance?.id);
      isInstanceRunning = !!found;
      instanceResourceUsage = found || null;

      if (found && (activeTab === 'logs' || activeTab === 'overview')) {
        loadLogsForInstance(currentInstance.folder);
      }
    } catch (e) {
      console.warn('Error verificando estado de ejecución:', e);
    }
  }

  async function handleTerminateById(instId: string) {
    try {
      await invoke('terminate_instance', { instanceId: instId });
      await checkRunningState();
    } catch (e) {
      alert(`Error al detener la instancia: ${e}`);
    }
  }

  let launchingInstanceId = $state<string | null>(null);

  async function handleLaunchInstance(instId: string) {
    if (launchingInstanceId) return;
    launchingInstanceId = instId;
    isLaunching = true;
    try {
      const inst = instances.find(i => i.id === instId);
      const instName = inst?.name || instId;
      const pid = await invoke<number>('launch_instance_cmd', {
        instanceId: instId,
        username: activeAccount.username,
        uuid: activeAccount.uuid || null,
      });
      console.log(`Instancia ${instName} iniciada con PID ${pid}`);
      await checkRunningState();
    } catch (e) {
      alert(`Error al iniciar la instancia: ${e}`);
    } finally {
      setTimeout(() => {
        launchingInstanceId = null;
        isLaunching = false;
      }, 1500);
    }
  }

  async function handleLaunch() {
    if (!currentInstance?.id) return;
    await handleLaunchInstance(currentInstance.id);
  }

  async function handleTerminateInstance() {
    try {
      await invoke('terminate_instance', { instanceId: currentInstance.id });
      isInstanceRunning = false;
      instanceResourceUsage = null;
    } catch (e) {
      alert(`Error al detener la instancia: ${e}`);
    }
  }

  async function handleSaveInstanceModal(formData: any) {
    try {
      if (instanceModalMode === 'create') {
        const newId = await invoke<string>('create_instance_cmd', {
          name: formData.name,
          mcVersion: formData.mc_version,
          loaderType: formData.loader_type,
          icon: formData.icon,
          executableType: formData.executable_type,
          launcherJar: formData.launcher_jar,
          customExec: formData.custom_exec,
          javaMode: formData.java_mode,
          customJava: formData.custom_java,
          ramMin: formData.ram_min,
          ramMax: formData.ram_max,
          jvmPreset: formData.jvm_preset,
          jvmArgs: formData.jvm_args,
          instanceType: formData.instance_type,
          serverPort: formData.server_port,
          assignedUsername: formData.assigned_username || null,
        });
        await loadDataFromRust();
        selectedInstanceId = newId;
      } else {
        await invoke('update_instance_cmd', {
          id: formData.id,
          name: formData.name,
          icon: formData.icon,
          mcVersion: formData.mc_version,
          loaderType: formData.loader_type,
          executableType: formData.executable_type,
          launcherJar: formData.launcher_jar,
          customExec: formData.custom_exec,
          javaMode: formData.java_mode,
          customJava: formData.custom_java,
          ramMin: formData.ram_min,
          ramMax: formData.ram_max,
          jvmPreset: formData.jvm_preset,
          jvmArgs: formData.jvm_args,
          width: 1280,
          height: 720,
          assignedUsername: formData.assigned_username || null,
        });
        await loadDataFromRust();
      }
      isInstanceModalOpen = false;
    } catch (e) {
      alert(`Error al guardar instancia: ${e}`);
    }
  }

  async function handleDeleteInstance(id: string) {
    if (!confirm('¿Estás seguro de eliminar esta instancia y todos sus archivos?')) return;
    try {
      await invoke('delete_instance_cmd', { id });
      await loadDataFromRust();
      if (instances.length > 0) {
        selectedInstanceId = instances[0].id;
      }
    } catch (e) {
      alert(`Error al eliminar: ${e}`);
    }
  }

  async function handleOpenFolder(folderName: string, subfolder?: string) {
    try {
      await invoke('open_folder_cmd', { folderName, subfolder: subfolder || null });
    } catch (e) {
      console.error(e);
    }
  }

  async function handleSetAccount(username: string, accountType: string, uuid?: string) {
    try {
      await invoke('set_account_cmd', { username, accountType, uuid: uuid || null });
      await loadDataFromRust();
      isAccountModalOpen = false;
    } catch (e) {
      alert(`Error de cuenta: ${e}`);
    }
  }

  async function handleDeleteAccount(id: string) {
    if (!confirm('¿Eliminar esta cuenta guardada en Omnicraft?')) return;
    try {
      await invoke('delete_account_cmd', { id });
      await loadDataFromRust();
    } catch (e) {
      alert(`Error al eliminar cuenta: ${e}`);
    }
  }

  async function handleLogoutUser(username: string) {
    if (!confirm(`¿Cerrar sesión de '${username}' y limpiar todos sus datos y texturas vinculadas en el sistema?`)) return;
    try {
      await invoke('logout_user_cmd', { username });
      await loadDataFromRust();
      alert(`✅ Sesión cerrada y registros de '${username}' eliminados de Omnicraft.`);
    } catch (e) {
      alert(`Error al cerrar sesión: ${e}`);
    }
  }

  onMount(() => {
    loadDataFromRust();
  });
</script>

<div class="flex h-screen w-screen bg-obsidian-950 text-slate-100 overflow-hidden font-sans">
  <!-- SIDEBAR IZQUIERDA -->
  <aside class="w-64 bg-obsidian-900/95 border-r border-obsidian-750/70 flex flex-col backdrop-blur-md">
    <!-- Brand Header -->
    <div class="p-4 flex items-center justify-between border-b border-obsidian-750/60">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-omni-indigo to-omni-cyan flex items-center justify-center font-black text-xl shadow-lg shadow-omni-indigo/25">
          Ω
        </div>
        <div>
          <h1 class="text-sm font-bold tracking-tight text-white flex items-center gap-1.5">
            Omnicraft <span class="px-1.5 py-0.5 rounded text-[10px] bg-omni-indigo/20 text-omni-lavender font-mono border border-omni-indigo/30">Studio</span>
          </h1>
          <p class="text-[11px] text-slate-400 font-medium">Multi-Minecraft Manager</p>
        </div>
      </div>
    </div>

    <!-- Selector de Instancia Activa y Estado en Sidebar Superior -->
    <div class="p-3 border-b border-obsidian-750/50 space-y-2 bg-obsidian-950/40">
      <div class="flex items-center justify-between">
        <span class="text-[10px] font-bold tracking-wider text-slate-400 uppercase">Instancia Activa</span>
        <button
          onclick={() => { instanceModalMode = 'create'; isInstanceModalOpen = true; }}
          class="text-[10px] flex items-center gap-1 font-bold text-emerald-400 hover:text-emerald-300 px-1.5 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20"
          title="Crear nueva instancia"
        >
          <Plus class="w-3 h-3" /> Nueva
        </button>
      </div>

      <div class="flex items-center gap-2 bg-obsidian-900 border border-obsidian-750 p-2 rounded-xl">
        <img
          src={`/minecraft-icons/${currentInstance.icon || 'art-Crafting_Table.png'}`}
          alt=""
          class="w-6 h-6 object-contain shrink-0"
          onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/minecraft-icons/art-Crafting_Table.png'; }}
        />
        <div class="min-w-0 flex-1">
          {#if instances.length > 0}
            <select
              value={selectedInstanceId}
              onchange={(e) => handleInstanceChange(e.currentTarget.value)}
              class="w-full bg-transparent text-xs font-bold text-white focus:outline-none cursor-pointer truncate border-none p-0"
            >
              {#each instances as inst}
                <option value={inst.id} class="bg-obsidian-900 text-white">{inst.name}</option>
              {/each}
            </select>
            <div class="flex items-center gap-1.5 mt-0.5 text-[10px] font-mono text-slate-400">
              <span class="text-emerald-400 font-semibold">{currentInstance.mc_version}</span>
              <span>•</span>
              <span class="text-indigo-400">{currentInstance.loader_type}</span>
            </div>
          {:else}
            <span class="text-xs font-bold text-amber-400">Sin instancias</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Perfil de Jugador en Sidebar -->
    <div class="p-3 border-b border-obsidian-750/40 space-y-2">
      <button
        onclick={() => isAccountModalOpen = true}
        class="w-full p-2 rounded-xl bg-obsidian-800/60 hover:bg-obsidian-800 border border-obsidian-750/70 flex items-center justify-between transition text-left"
        title="Gestionar Cuentas"
      >
        <div class="flex items-center gap-2.5">
          <img
            src={SkinService.getAvatarUrl(activeAccount.username, 28)}
            alt=""
            class="w-7 h-7 rounded-lg border border-obsidian-700 object-cover"
            onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/minecraft-icons/mini-steve.png'; }}
          />
          <div class="min-w-0">
            <p class="text-xs font-bold text-white truncate">{activeAccount.username}</p>
            <p class="text-[10px] text-slate-400 font-mono truncate">{activeAccount.account_type === 'microsoft' ? 'Microsoft Oficial' : 'Non-Premium'}</p>
          </div>
        </div>
        <span class="text-[10px] text-omni-lavender font-bold px-1.5 py-0.5 bg-omni-indigo/20 rounded">Cuentas</span>
      </button>

      <button
        onclick={() => isSkinEditorModalOpen = true}
        class="w-full py-1.5 px-2.5 rounded-lg bg-gradient-to-r from-omni-indigo/20 to-omni-cyan/20 hover:from-omni-indigo/30 hover:to-omni-cyan/30 border border-omni-cyan/30 flex items-center justify-between transition text-left"
        title="Diseñar, editar y exportar skins (Compatibilidad Offline y Mojang)"
      >
        <div class="flex items-center gap-2">
          <Sparkles class="w-3.5 h-3.5 text-omni-cyan" />
          <span class="text-xs font-bold text-omni-cyan">Editor de Skins</span>
        </div>
        <span class="text-[10px] bg-omni-cyan/20 text-omni-cyan font-mono px-1 rounded">{localSkins.length}</span>
      </button>
    </div>

    <!-- Navegación Principal Organizada Semánticamente -->
    <nav class="flex-1 p-3 space-y-4 overflow-y-auto">
      <!-- Sección Núcleo de Juego -->
      <div class="space-y-1">
        <span class="text-[10px] font-bold tracking-wider text-slate-400 uppercase px-3">{t('section_instances')}</span>
        <button
          onclick={() => activeTab = 'overview'}
          class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'overview' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
        >
          <Activity class="w-4 h-4 text-emerald-400" />
          <span>{t('nav_overview')}</span>
        </button>

        <button
          onclick={() => activeTab = 'instances'}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'instances' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
        >
          <div class="flex items-center gap-3">
            <Layers class="w-4 h-4 text-omni-cyan" />
            <span>{t('nav_instances')}</span>
          </div>
          <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-950/80 font-mono text-slate-300">{instances.length}</span>
        </button>
      </div>

      <!-- Sección Contenido & Descargas -->
      <div class="space-y-1">
        <span class="text-[10px] font-bold tracking-wider text-slate-400 uppercase px-3">{t('section_content')}</span>
        <button
          onclick={() => activeTab = 'store'}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'store' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
        >
          <div class="flex items-center gap-3">
            <Download class="w-4 h-4 text-amber-400" />
            <span>{t('nav_store')}</span>
          </div>
          <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-950/80 font-mono text-slate-300">{searchResults.length}</span>
        </button>

        <button
          onclick={() => activeTab = 'mods'}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'mods' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
        >
          <div class="flex items-center gap-3">
            <Package class="w-4 h-4 text-teal-400" />
            <span>{t('nav_mods')}</span>
          </div>
          <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-950/80 font-mono text-slate-300">{contentItems.length}</span>
        </button>

        <!-- Botón Verificador de Compatibilidad -->
        <button
          onclick={() => { activeTab = 'mods'; runCompatibilityCheck(); }}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all text-slate-300 hover:text-white hover:bg-obsidian-800"
        >
          <div class="flex items-center gap-3">
            <Shield class="w-4 h-4 {compatibilityReport && !compatibilityReport.is_compatible ? 'text-red-400 animate-pulse' : 'text-emerald-400'}" />
            <span>Compatibilidad</span>
          </div>
          {#if compatibilityReport}
            {#if compatibilityReport.issues.length > 0}
              <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-red-500/20 text-red-300 font-bold border border-red-500/30">{compatibilityReport.issues.length}</span>
            {:else}
              <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 font-bold border border-emerald-500/30">100% OK</span>
            {/if}
          {/if}
        </button>
      </div>

      <!-- Sección Suite de Creadores & Streamers (Solo en Modo Avanzado) -->
      {#if uiMode === 'advanced'}
        <div class="space-y-1">
          <span class="text-[10px] font-bold tracking-wider text-amber-400 uppercase px-3 flex items-center gap-1.5">
            <Cpu class="w-3 h-3" /> {t('section_advanced')}
          </span>
          <button
            onclick={() => activeTab = 'loaders'}
            class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'loaders' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
          >
            <Server class="w-4 h-4 text-indigo-400" />
            <span>{t('nav_loaders')}</span>
          </button>

          <button
            onclick={() => activeTab = 'logs'}
            class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'logs' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
          >
            <div class="flex items-center gap-3">
              <Terminal class="w-4 h-4 text-emerald-400" />
              <span>{t('nav_logs')}</span>
            </div>
            <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-300 font-mono border border-emerald-500/40">Vivo</span>
          </button>
        </div>
      {/if}

      <!-- Mundos & Backups (Accesible siempre) -->
      <div class="space-y-1">
        <span class="text-[10px] font-bold tracking-wider text-slate-400 uppercase px-3">{t('nav_worlds')}</span>
        <button
          onclick={() => activeTab = 'worlds'}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'worlds' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
        >
          <div class="flex items-center gap-3">
            <Globe class="w-4 h-4 text-sky-400" />
            <span>{t('nav_worlds')}</span>
          </div>
          <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-950/80 font-mono text-slate-300">{instanceWorlds.length}</span>
        </button>
      </div>

      <!-- Sección Sistema -->
      <div class="space-y-1">
        <span class="text-[10px] font-bold tracking-wider text-slate-400 uppercase px-3">{t('section_system')}</span>
        <button
          onclick={() => activeTab = 'config'}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all {activeTab === 'config' ? 'bg-omni-indigo text-white shadow-md shadow-omni-indigo/20' : 'text-slate-300 hover:text-white hover:bg-obsidian-800'}"
        >
          <div class="flex items-center gap-3">
            <Settings class="w-4 h-4 text-slate-400" />
            <span>{t('nav_config')}</span>
          </div>
          <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-950/80 font-mono text-slate-300">{javas.length}</span>
        </button>

        <!-- Apoyo / Donaciones en Sidebar -->
        <button
          type="button"
          onclick={() => openExternalUrl('https://github.com/correakarl/omnicraft-studio')}
          class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-semibold transition-all text-pink-300 hover:text-white hover:bg-pink-900/30 border border-pink-500/20 bg-pink-950/20 text-left"
          title="Ver proyecto y apoyar en GitHub / PayPal"
        >
          <div class="flex items-center gap-3">
            <Heart class="w-4 h-4 text-pink-400 fill-pink-400/50" />
            <span>Donar al Proyecto</span>
          </div>
          <span class="text-[10px] text-pink-400 font-mono">GitHub</span>
        </button>
      </div>
    </nav>

    <!-- Estado del Sistema en Footer -->
    <div class="p-3 m-3 rounded-xl bg-obsidian-800/60 border border-obsidian-750/80">
      <div class="flex items-center gap-2 text-xs font-semibold text-emerald-400">
        <CheckCircle2 class="w-3.5 h-3.5" />
        <span>{t('sqlite_connected', 'Sistema Conectado')}</span>
      </div>
      <p class="text-[10px] text-slate-400 mt-1 truncate">{coreStatus}</p>
    </div>
  </aside>

  <!-- CONTENIDO PRINCIPAL -->
  <main class="flex-1 flex flex-col bg-obsidian-950/95 overflow-hidden">
    <!-- BARRA SUPERIOR LIMPIA Y ESPACIOSA -->
    <header class="h-14 px-4 bg-obsidian-900/80 border-b border-obsidian-750/70 flex items-center justify-between backdrop-blur-md gap-3 shrink-0">
      <!-- Lado Izquierdo: Selector de Instancia Predeterminada / Atajos de Carpeta y Ajustes -->
      <div class="flex items-center gap-2 min-w-0">
        {#if instances.length > 0}
          <div class="flex items-center bg-obsidian-950/80 border border-obsidian-800 rounded-lg px-2.5 py-1 gap-2 shadow-inner">
            <span class="w-2 h-2 rounded-full {isLaunching ? 'bg-amber-400 animate-ping' : (isInstanceRunning ? 'bg-emerald-400' : 'bg-slate-500')}"></span>
            
            <span class="text-[10px] uppercase font-bold text-slate-400 tracking-wider">Default:</span>
            <select
              value={selectedInstanceId}
              onchange={(e) => handleInstanceChange(e.currentTarget.value)}
              class="bg-transparent text-xs font-bold text-white focus:outline-none cursor-pointer border-none p-0 pr-1 max-w-[140px] truncate"
              title="Instancia seleccionada para arrancar desde el botón principal"
            >
              {#each instances as inst}
                <option value={inst.id} class="bg-obsidian-900 text-white font-medium">
                  {inst.name}
                </option>
              {/each}
            </select>

            <span class="text-[10px] px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-mono font-medium">
              {currentInstance.mc_version}
            </span>
          </div>

          <div class="flex items-center gap-1">
            <button
              onclick={() => handleOpenFolder(currentInstance.folder)}
              class="p-1.5 rounded-lg bg-obsidian-950/60 hover:bg-obsidian-800 text-slate-400 hover:text-amber-400 border border-obsidian-800 transition"
              title="Abrir carpeta .minecraft"
            >
              <FolderOpen class="w-3.5 h-3.5" />
            </button>
            <button
              onclick={() => { instanceModalMode = 'edit'; isInstanceModalOpen = true; }}
              class="p-1.5 rounded-lg bg-obsidian-950/60 hover:bg-obsidian-800 text-slate-400 hover:text-omni-cyan border border-obsidian-800 transition"
              title="Ajustes de la instancia"
            >
              <Edit3 class="w-3.5 h-3.5" />
            </button>
          </div>
        {:else}
          <div class="text-xs text-slate-400">Sin instancias creadas</div>
        {/if}
      </div>

      <!-- Lado Derecho: Idioma, Descargas, Modo UI y Botón Jugar -->
      <div class="flex items-center gap-2 shrink-0">
        <!-- Selector de Idioma Compacto -->
        <div class="flex items-center bg-obsidian-950/80 border border-obsidian-800 rounded-lg px-2 py-1 text-xs">
          <Languages class="w-3.5 h-3.5 text-omni-indigo mr-1 shrink-0" />
          <select
            value={currentLang}
            onchange={(e) => handleLanguageChange(e.currentTarget.value)}
            class="bg-transparent text-xs font-bold text-slate-300 focus:outline-none cursor-pointer border-none p-0 pr-1"
            title="Seleccionar Idioma / Select Language"
          >
            {#each availableLanguages as lang}
              <option value={lang.code} class="bg-obsidian-900 text-white">
                {lang.flag} {lang.name}
              </option>
            {/each}
          </select>
        </div>

        <!-- Botón Cajón de Descargas -->
        <button
          onclick={() => showDownloadsDrawer = !showDownloadsDrawer}
          class="p-1.5 rounded-lg border text-xs font-bold transition shadow-sm bg-obsidian-950/80 border-obsidian-800 text-slate-300 hover:text-white hover:border-omni-indigo flex items-center gap-1.5"
          title="Ver Cola de Descargas Activa"
        >
          <Download class="w-3.5 h-3.5 {downloadTasks.some(t => !t.completed) ? 'text-amber-400 animate-bounce' : 'text-slate-400'}" />
          {#if downloadTasks.length > 0}
            <span class="px-1.5 py-0.2 rounded-full text-[9px] font-bold {downloadTasks.some(t => !t.completed) ? 'bg-amber-500 text-black' : 'bg-obsidian-800 text-slate-300'}">
              {downloadTasks.length}
            </span>
          {/if}
        </button>

        <!-- Switch de Modo Básico / Modo Avanzado -->
        <button
          onclick={toggleUiMode}
          class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-bold transition shadow-sm {uiMode === 'advanced' ? 'bg-amber-500/15 border-amber-500/40 text-amber-300' : 'bg-emerald-500/15 border-emerald-500/40 text-emerald-300'}"
          title="Alternar entre Modo Básico y Modo Avanzado"
        >
          {#if uiMode === 'advanced'}
            <Cpu class="w-3.5 h-3.5 text-amber-400" />
            <span>{t('mode_advanced')}</span>
          {:else}
            <Sparkles class="w-3.5 h-3.5 text-emerald-400" />
            <span>{t('mode_basic')}</span>
          {/if}
        </button>

        <!-- Actualizaciones Si Aplican -->
        {#if appUpdate?.has_update}
          <button
            onclick={() => isUpdateModalOpen = true}
            class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-indigo-600/30 hover:bg-indigo-600/50 text-omni-lavender border border-indigo-400/50 text-xs font-bold animate-pulse transition"
            title="Nueva versión de Omnicraft disponible"
          >
            <Download class="w-3.5 h-3.5 text-omni-cyan" />
            <span>Update</span>
          </button>
        {/if}

        <!-- BOTÓN PRINCIPAL JUGAR MINECRAFT / ESTADO DE EJECUCIÓN -->
        {#if isInstanceRunning}
          <div class="flex items-center gap-1.5 ml-1">
            <div class="flex items-center gap-2 px-2.5 py-1 rounded-lg bg-emerald-500/15 border border-emerald-500/40 text-emerald-400 text-xs font-bold animate-pulse">
              <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
              <span>Ejecutando</span>
            </div>
            <button
              onclick={handleTerminateInstance}
              class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg font-bold text-xs bg-rose-600/20 hover:bg-rose-600/40 text-rose-300 border border-rose-500/40 transition active:scale-95"
              title="Detener el proceso del juego"
            >
              <XCircle class="w-3.5 h-3.5" />
              <span>Detener</span>
            </button>
          </div>
        {:else}
          <button
            onclick={handleLaunch}
            disabled={isLaunching || instances.length === 0}
            class="flex items-center gap-2 px-4 py-1.5 rounded-lg font-bold text-xs tracking-wide transition-all shadow-md bg-gradient-to-r from-emerald-500 to-teal-400 hover:brightness-110 text-obsidian-950 shadow-emerald-500/20 active:scale-95 disabled:opacity-50 ml-1"
            title={currentInstance.executable_type === 'direct' ? 'Ejecutar cliente de forma nativa' : 'Arrancar instancia'}
          >
            {#if isLaunching}
              <RotateCw class="w-3.5 h-3.5 animate-spin" />
              <span>{t('starting')}</span>
            {:else}
              <Play class="w-3.5 h-3.5 fill-current" />
              <span>Jugar Ahora</span>
            {/if}
          </button>
        {/if}
      </div>
    </header>

    <!-- ÁREA DE CONTENIDO DINÁMICO -->
    <div class="flex-1 p-6 overflow-y-auto">
      {#if activeTab === 'overview'}
        <div class="space-y-6 max-w-6xl">
          <!-- Monitor de Recursos Multi-Instancia en Vivo si hay instancias en ejecución -->
          {#if allRunningUsages.length > 0}
            <div class="space-y-2">
              <div class="flex items-center justify-between text-xs font-bold text-slate-300">
                <span class="flex items-center gap-2">
                  <Activity class="w-4 h-4 text-emerald-400 animate-pulse" />
                  Instancias de Minecraft en Ejecución Simultánea ({allRunningUsages.length})
                </span>
                <span class="text-[11px] text-slate-400 font-normal">Supervisión de procesos y RAM en tiempo real</span>
              </div>

              {#each allRunningUsages as runUsage}
                <div class="p-3.5 rounded-2xl bg-gradient-to-r from-emerald-950/40 via-obsidian-900/80 to-obsidian-900/60 border border-emerald-500/30 shadow-md flex flex-col md:flex-row items-center justify-between gap-3">
                  <div class="flex items-center gap-3">
                    <div class="relative">
                      <div class="w-9 h-9 rounded-xl bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-400">
                        <Activity class="w-4 h-4 animate-pulse" />
                      </div>
                      <span class="absolute -top-1 -right-1 flex h-2.5 w-2.5">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                      </span>
                    </div>
                    <div>
                      <div class="flex items-center gap-2">
                        <span class="font-bold text-sm text-white">{runUsage.instance_name || runUsage.instance_id}</span>
                        <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                          PID: {runUsage.pid}
                        </span>
                        {#if runUsage.username}
                          <span class="px-2 py-0.5 rounded text-[10px] font-mono bg-omni-indigo/20 text-omni-lavender border border-omni-indigo/30 flex items-center gap-1">
                            <span>👤</span> {runUsage.username}
                          </span>
                        {/if}
                      </div>
                      <p class="text-[11px] text-slate-400">Motor Nativo Directo • Aislamiento independiente</p>
                    </div>
                  </div>

                  <div class="flex items-center gap-5 w-full md:w-auto justify-between md:justify-end">
                    <div class="text-left md:text-right">
                      <span class="text-[9px] font-bold uppercase tracking-wider text-slate-400 block">CPU</span>
                      <span class="text-sm font-mono font-bold text-emerald-400">{runUsage.cpu_percent.toFixed(1)}%</span>
                    </div>

                    <div class="text-left md:text-right">
                      <span class="text-[9px] font-bold uppercase tracking-wider text-slate-400 block">RAM Asignada</span>
                      <span class="text-sm font-mono font-bold text-teal-300">
                        {runUsage.ram_used_mb} MB <span class="text-[10px] text-slate-500 font-normal">/ {runUsage.total_system_ram_mb} MB</span>
                      </span>
                    </div>

                    <button
                      onclick={() => runUsage.instance_id && handleTerminateById(runUsage.instance_id)}
                      class="px-3 py-1.5 rounded-xl bg-rose-600/20 hover:bg-rose-600/30 text-rose-300 border border-rose-500/30 font-bold text-xs transition flex items-center gap-1.5"
                      title="Detener este proceso"
                    >
                      <XCircle class="w-3.5 h-3.5 text-rose-400" />
                      <span>Detener</span>
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Tarjetas de métricas -->
          <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div class="p-4 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 shadow-sm">
              <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-omni-indigo" /> {t('nav_instances')}
              </span>
              <div class="mt-2 text-2xl font-bold text-white">{instances.length}</div>
              <p class="text-[11px] text-slate-500 mt-1">{t('filter_all')}</p>
            </div>

            <div class="p-4 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 shadow-sm">
              <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Cpu class="w-3.5 h-3.5 text-emerald-400" /> {t('stats_memory')}
              </span>
              <div class="mt-2 text-2xl font-bold text-emerald-400">{(currentInstance.ram_max_mb / 1024).toFixed(1)} GB</div>
              <p class="text-[11px] text-slate-500 mt-1">Preset: {currentInstance.jvm_preset || 'Aikar Flags'}</p>
            </div>

            <div class="p-4 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 shadow-sm">
              <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Package class="w-3.5 h-3.5 text-omni-cyan" /> {t('nav_mods')}
              </span>
              <div class="mt-2 text-2xl font-bold text-white">{instanceMods.length}</div>
              <p class="text-[11px] text-slate-500 mt-1">{currentInstance.loader_type} {currentInstance.mc_version}</p>
            </div>

            <div class="p-4 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 shadow-sm">
              <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Coffee class="w-3.5 h-3.5 text-amber-400" /> {t('stats_java')}
              </span>
              <div class="mt-2 text-2xl font-bold text-amber-400">{javas.length} versiones</div>
              <p class="text-[11px] text-slate-500 mt-1">Temurin / OpenJDK</p>
            </div>
          </div>

          <!-- Acceso Rápido a Carpetas de Juego Personalizadas -->
          <div class="p-5 rounded-2xl bg-obsidian-900/60 border border-obsidian-750/80">
            <h2 class="text-sm font-bold text-white mb-1.5 flex items-center gap-2">
              <FolderOpen class="w-4 h-4 text-amber-400" />
              {t('open_folder')} ({currentInstance.name})
            </h2>
            <p class="text-xs text-slate-400 mb-4">
              Cada instancia contiene su propio entorno atómico de archivos para no alterar tus otros perfiles.
            </p>

            <div class="grid grid-cols-2 md:grid-cols-5 gap-3">
              <button
                onclick={() => handleOpenFolder(currentInstance.folder, 'mods')}
                class="p-3 rounded-xl bg-obsidian-950/70 border border-obsidian-800 hover:border-omni-indigo transition text-left"
              >
                <p class="font-bold text-xs text-white">📁 mods/</p>
                <p class="text-[10px] text-slate-400 mt-0.5">Archivos .jar</p>
              </button>

              <button
                onclick={() => handleOpenFolder(currentInstance.folder, 'shaderpacks')}
                class="p-3 rounded-xl bg-obsidian-950/70 border border-obsidian-800 hover:border-omni-indigo transition text-left"
              >
                <p class="font-bold text-xs text-white">✨ shaderpacks/</p>
                <p class="text-[10px] text-slate-400 mt-0.5">Shaders Iris/OptiFine</p>
              </button>

              <button
                onclick={() => handleOpenFolder(currentInstance.folder, 'resourcepacks')}
                class="p-3 rounded-xl bg-obsidian-950/70 border border-obsidian-800 hover:border-omni-indigo transition text-left"
              >
                <p class="font-bold text-xs text-white">🎨 resourcepacks/</p>
                <p class="text-[10px] text-slate-400 mt-0.5">Paquetes de texturas</p>
              </button>

              <button
                onclick={() => handleOpenFolder(currentInstance.folder, 'saves')}
                class="p-3 rounded-xl bg-obsidian-950/70 border border-obsidian-800 hover:border-omni-indigo transition text-left"
              >
                <p class="font-bold text-xs text-white">🗺️ saves/</p>
                <p class="text-[10px] text-slate-400 mt-0.5">Mundos guardados</p>
              </button>

              <button
                onclick={() => handleOpenFolder(currentInstance.folder, 'config')}
                class="p-3 rounded-xl bg-obsidian-950/70 border border-obsidian-800 hover:border-omni-indigo transition text-left"
              >
                <p class="font-bold text-xs text-white">⚙️ config/</p>
                <p class="text-[10px] text-slate-400 mt-0.5">Ajustes de mods</p>
              </button>
            </div>
          </div>
        </div>

      {:else if activeTab === 'instances'}
        <!-- PESTAÑA ADMINISTRAR INSTANCIAS -->
        <div class="space-y-6 max-w-6xl">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-bold text-white flex items-center gap-2">
                <Layers class="w-5 h-5 text-omni-cyan" />
                <span>Instancias Creadas ({instances.length})</span>
              </h2>
              <p class="text-xs text-slate-400">
                Ejecuta individualmente cada instancia, supervisa sus recursos en vivo (CPU/RAM) y asigna cuál arranca por defecto desde el navbar.
              </p>
            </div>
            <button
              onclick={() => { instanceModalMode = 'create'; isInstanceModalOpen = true; }}
              class="flex items-center gap-2 px-4 py-2 rounded-xl bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold text-xs transition shadow-md shadow-omni-indigo/25"
            >
              <Plus class="w-4 h-4" />
              <span>{t('create_instance')}</span>
            </button>
          </div>

          <!-- PANEL DE MONITOREO DE TODAS LAS INSTANCIAS EN EJECUCIÓN ACTIVA -->
          {#if allRunningUsages.length > 0}
            <div class="p-4 rounded-2xl bg-gradient-to-br from-emerald-950/40 via-obsidian-900/80 to-obsidian-900/90 border border-emerald-500/30 shadow-lg space-y-3">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 rounded-full bg-emerald-400 animate-ping"></div>
                  <h3 class="text-xs font-bold uppercase tracking-wider text-emerald-300">
                    Monitor de Recursos Multi-Instancia en Vivo ({allRunningUsages.length} activas)
                  </h3>
                </div>
                <span class="text-[11px] text-slate-400 font-mono">Actualizado cada 1.5s</span>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each allRunningUsages as rUsage}
                  <div class="p-3 rounded-xl bg-obsidian-950/80 border border-emerald-500/20 flex items-center justify-between gap-3 shadow-inner">
                    <div class="min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="font-bold text-xs text-white truncate">{rUsage.instance_name || rUsage.instance_id}</span>
                        <span class="text-[10px] px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-300 font-mono border border-emerald-500/30">
                          PID {rUsage.pid}
                        </span>
                      </div>
                      <div class="flex items-center gap-2 mt-1 text-[11px] text-slate-400 font-mono">
                        <span>👤 {rUsage.username || 'Steve'}</span>
                        <span>•</span>
                        <span class="text-emerald-400 font-bold">{rUsage.cpu_percent.toFixed(1)}% CPU</span>
                        <span>•</span>
                        <span class="text-teal-300 font-bold">{rUsage.ram_used_mb} MB RAM</span>
                      </div>
                    </div>

                    <button
                      onclick={() => rUsage.instance_id && handleTerminateById(rUsage.instance_id)}
                      class="px-2.5 py-1.5 rounded-lg bg-rose-600/20 hover:bg-rose-600/40 text-rose-300 border border-rose-500/30 font-bold text-xs transition flex items-center gap-1 shrink-0"
                      title="Detener este proceso"
                    >
                      <XCircle class="w-3.5 h-3.5" />
                      <span>Detener</span>
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- LISTADO DE INSTANCIAS CON ACCIONES Y CONTROL DE EJECUCIÓN -->
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each instances as inst}
              {@const isThisRunning = allRunningUsages.some(u => u.instance_id === inst.id)}
              {@const thisUsage = allRunningUsages.find(u => u.instance_id === inst.id)}
              {@const isDefault = inst.id === selectedInstanceId}
              
              <div class="p-4 rounded-2xl bg-obsidian-900/60 border transition-all {isThisRunning ? 'border-emerald-500/60 shadow-lg shadow-emerald-500/10' : (isDefault ? 'border-omni-cyan/80 shadow-md shadow-omni-cyan/10' : 'border-obsidian-750/70 hover:border-obsidian-600')} flex flex-col justify-between">
                <div>
                  <div class="flex items-start gap-3">
                    <img
                      src={`/minecraft-icons/${inst.icon || 'art-Crafting_Table.png'}`}
                      alt=""
                      class="w-10 h-10 object-contain p-1 rounded-xl bg-obsidian-950 border border-obsidian-800 shrink-0"
                      onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/minecraft-icons/art-Crafting_Table.png'; }}
                    />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-1.5">
                        <h3 class="text-sm font-bold text-white truncate max-w-[130px]">{inst.name}</h3>
                        {#if isDefault}
                          <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-omni-cyan/20 text-omni-cyan border border-omni-cyan/30 shrink-0">
                            DEFAULT
                          </span>
                        {/if}
                        {#if isThisRunning}
                          <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1 shrink-0 animate-pulse">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span> ACTIVA
                          </span>
                        {/if}
                      </div>

                      <p class="text-[11px] text-slate-400 font-mono mt-0.5 truncate">
                        👤 {inst.assigned_username || 'Heredar'} • {inst.instance_type === 'server' ? 'Servidor' : 'Cliente'}
                      </p>
                    </div>
                  </div>

                  <!-- Métricas y configuración de la tarjeta -->
                  <div class="grid grid-cols-2 gap-2 mt-3 text-[11px]">
                    <div class="p-2 rounded-lg bg-obsidian-950/60 border border-obsidian-800">
                      <span class="text-slate-400 block text-[10px]">Versión & Loader</span>
                      <span class="font-bold text-white truncate block">{inst.mc_version} {inst.loader_type}</span>
                    </div>
                    <div class="p-2 rounded-lg bg-obsidian-950/60 border border-obsidian-800">
                      <span class="text-slate-400 block text-[10px]">Memoria RAM</span>
                      <span class="font-bold text-emerald-400 truncate block">{(inst.ram_min_mb / 1024).toFixed(1)}G - {(inst.ram_max_mb / 1024).toFixed(1)}G</span>
                    </div>
                  </div>

                  <!-- Estado en vivo si está corriendo -->
                  {#if isThisRunning && thisUsage}
                    <div class="mt-2.5 p-2 rounded-lg bg-emerald-950/30 border border-emerald-500/30 flex items-center justify-between text-[11px] font-mono text-emerald-300">
                      <span>PID {thisUsage.pid}</span>
                      <span>CPU {thisUsage.cpu_percent.toFixed(1)}%</span>
                      <span>{thisUsage.ram_used_mb} MB</span>
                    </div>
                  {:else}
                    <div class="mt-2 text-[10px] text-slate-500 flex items-center justify-between">
                      <span class="truncate">Motor: <strong class="text-slate-400">{inst.executable_type}</strong></span>
                      <span class="truncate">{inst.launcher_jar || 'Nativo Directo'}</span>
                    </div>
                  {/if}
                </div>

                <!-- Botones de Acción de Instancia -->
                <div class="mt-3.5 pt-3 border-t border-obsidian-800 flex flex-col gap-2">
                  {#if inst.instance_type === 'server'}
                    <button
                      onclick={handleDownloadPaperServer}
                      disabled={isDownloadingServer}
                      class="w-full py-1.5 px-2 rounded-lg bg-amber-500/15 hover:bg-amber-500/25 text-amber-300 border border-amber-500/30 text-[11px] font-bold transition flex items-center justify-center gap-1.5"
                    >
                      {#if isDownloadingServer}
                        <RotateCw class="w-3 h-3 animate-spin" /> Descargando Paper...
                      {:else}
                        <Download class="w-3 h-3 text-amber-400" /> Descargar PaperMC Server Oficial
                      {/if}
                    </button>
                  {/if}

                  <!-- Fila 1: Botones de Utilidad -->
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-1.5">
                      <button
                        onclick={() => handleOpenFolder(inst.folder)}
                        class="p-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-slate-300 transition"
                        title="Abrir carpeta de la instancia"
                      >
                        <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
                      </button>
                      <button
                        onclick={() => { selectedInstanceId = inst.id; instanceModalMode = 'edit'; isInstanceModalOpen = true; }}
                        class="p-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-slate-300 transition"
                        title="Editar configuración"
                      >
                        <Edit3 class="w-3.5 h-3.5 text-omni-cyan" />
                      </button>
                      <button
                        onclick={() => handleDeleteInstance(inst.id)}
                        disabled={isThisRunning}
                        class="p-1.5 rounded-lg bg-obsidian-800 hover:bg-red-500/20 text-slate-400 hover:text-red-400 transition disabled:opacity-30"
                        title="Eliminar instancia"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>

                    <!-- Botón Marcar Default -->
                    <button
                      onclick={() => handleInstanceChange(inst.id)}
                      class="px-2 py-1 rounded-lg text-[11px] font-semibold transition border {isDefault ? 'bg-omni-cyan/15 border-omni-cyan text-omni-cyan' : 'bg-obsidian-800/80 border-obsidian-750 text-slate-400 hover:text-white'}"
                      title={isDefault ? 'Esta instancia ya es la predeterminada del navbar' : 'Establecer como instancia por defecto en el navbar'}
                    >
                      {isDefault ? '★ Default' : 'Hacer Default'}
                    </button>
                  </div>

                  <!-- Fila 2: Botón de Ejecutar / Detener (Ancho completo, sin apretarse) -->
                  <div>
                    {#if isThisRunning}
                      <button
                        onclick={() => handleTerminateById(inst.id)}
                        class="w-full py-1.5 rounded-lg text-xs font-bold bg-rose-600/20 hover:bg-rose-600/30 text-rose-300 border border-rose-500/30 transition flex items-center justify-center gap-1.5"
                      >
                        <XCircle class="w-3.5 h-3.5" />
                        <span>Detener Instancia</span>
                      </button>
                    {:else}
                      <button
                        onclick={() => handleLaunchInstance(inst.id)}
                        disabled={launchingInstanceId === inst.id}
                        class="w-full py-1.5 rounded-lg text-xs font-bold transition shadow-sm flex items-center justify-center gap-1.5 bg-gradient-to-r from-emerald-500 to-teal-400 text-obsidian-950 hover:brightness-110 active:scale-95 disabled:opacity-50"
                      >
                        {#if launchingInstanceId === inst.id}
                          <RotateCw class="w-3.5 h-3.5 animate-spin" />
                          <span>Iniciando...</span>
                        {:else}
                          <Play class="w-3.5 h-3.5 fill-current" />
                          <span>Ejecutar Instancia</span>
                        {/if}
                      </button>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>

      {:else if activeTab === 'loaders'}
        <!-- PESTAÑA MOD LOADERS -->
        <div class="space-y-6 max-w-6xl">
          <div>
            <h2 class="text-base font-bold text-white">{t('nav_loaders')}</h2>
            <p class="text-xs text-slate-400">Elige e instala el cargador de mods óptimo para {currentInstance.name} (MC {currentInstance.mc_version}).</p>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div class="p-5 rounded-2xl bg-obsidian-900/60 border border-obsidian-750/70 flex flex-col justify-between">
              <div>
                <span class="px-2 py-0.5 rounded text-[10px] bg-indigo-500/20 text-indigo-400 font-bold">Ultra Rápido</span>
                <h3 class="text-base font-bold text-white mt-2">Fabric Loader</h3>
                <p class="text-xs text-slate-400 mt-1">El estándar moderno más ligero, ideal para shaders (Iris/Sodium) y optimización extrema de FPS.</p>
              </div>
              <button
                onclick={handleInstallFabric}
                disabled={isInstallingLoader}
                class="mt-4 w-full py-2 bg-omni-indigo hover:bg-omni-indigo/80 text-white rounded-xl font-bold text-xs transition flex items-center justify-center gap-2 disabled:opacity-50"
              >
                {#if isInstallingLoader}
                  <RotateCw class="w-3.5 h-3.5 animate-spin" /> Instalando...
                {:else}
                  <Download class="w-3.5 h-3.5" /> Descargar & Instalar Fabric Oficial
                {/if}
              </button>
            </div>

            <div class="p-5 rounded-2xl bg-obsidian-900/60 border border-obsidian-750/70 flex flex-col justify-between">
              <div>
                <span class="px-2 py-0.5 rounded text-[10px] bg-emerald-500/20 text-emerald-400 font-bold">Nueva Generación</span>
                <h3 class="text-base font-bold text-white mt-2">NeoForge</h3>
                <p class="text-xs text-slate-400 mt-1">El sucesor oficial de Forge a partir de MC 1.20.2+, altamente optimizado y compatible con modpacks extensos.</p>
              </div>
              <button
                onclick={() => alert('Configuración NeoForge aplicada.')}
                class="mt-4 w-full py-2 bg-obsidian-800 hover:bg-obsidian-750 text-white rounded-xl font-bold text-xs transition"
              >
                Cambiar a NeoForge
              </button>
            </div>

            <div class="p-5 rounded-2xl bg-obsidian-900/60 border border-obsidian-750/70 flex flex-col justify-between">
              <div>
                <span class="px-2 py-0.5 rounded text-[10px] bg-amber-500/20 text-amber-400 font-bold">Clásico</span>
                <h3 class="text-base font-bold text-white mt-2">Minecraft Forge</h3>
                <p class="text-xs text-slate-400 mt-1">El ecosistema legendario con soporte para miles de mods clásicos y versiones históricas.</p>
              </div>
              <button
                onclick={() => alert('Configuración Forge aplicada.')}
                class="mt-4 w-full py-2 bg-obsidian-800 hover:bg-obsidian-750 text-white rounded-xl font-bold text-xs transition"
              >
                Cambiar a Forge
              </button>
            </div>
          </div>
        </div>

      {:else if activeTab === 'mods'}
        <!-- PESTAÑA GESTOR Y EDITOR DE CONTENIDO DESCARGADO (MODS, SHADERS, RESOURCEPACKS) -->
        <div class="space-y-4 max-w-6xl">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-bold text-white flex items-center gap-2">
                <Package class="w-5 h-5 text-omni-indigo" />
                {t('nav_mods')} de {currentInstance.name}
              </h2>
              <p class="text-xs text-slate-400">
                Habilita, deshabilita o elimina mods, shaders y resourcepacks con un clic sin romper tus archivos.
              </p>
            </div>
            <div class="flex items-center gap-2">
              <button
                onclick={openSharedCacheModal}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-teal-500/10 hover:bg-teal-500/20 text-teal-300 border border-teal-500/30 text-xs font-bold transition shadow-sm"
                title="Vincular contenido ya descargado en otras instancias sin ocupar espacio en disco"
              >
                <Layers class="w-3.5 h-3.5 text-teal-400" />
                <span>Repositorio Compartido (0 MB Extra)</span>
              </button>
              <button
                onclick={() => loadContentForInstance(currentInstance.folder, activeContentCategory)}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-slate-300 text-xs font-medium transition border border-obsidian-700"
              >
                <RotateCw class="w-3.5 h-3.5" />
                <span>Refrescar</span>
              </button>
              <button
                onclick={() => handleOpenFolder(currentInstance.folder, activeContentCategory)}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-amber-400 text-xs font-medium transition border border-obsidian-700"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                <span>Abrir Carpeta</span>
              </button>
            </div>
          </div>

          <!-- Panel de Auditoría de Compatibilidad en Vivo -->
          {#if activeContentCategory === 'mods'}
            <div class="p-3.5 rounded-xl border transition-all {compatibilityReport && !compatibilityReport.is_compatible ? 'bg-rose-950/20 border-rose-600/40' : 'bg-emerald-950/20 border-emerald-600/40'} flex items-start justify-between gap-4">
              <div class="flex items-start gap-3">
                <Shield class="w-5 h-5 shrink-0 mt-0.5 {compatibilityReport && !compatibilityReport.is_compatible ? 'text-rose-400' : 'text-emerald-400'}" />
                <div>
                  <div class="flex items-center gap-2">
                    <h3 class="text-xs font-bold {compatibilityReport && !compatibilityReport.is_compatible ? 'text-rose-300' : 'text-emerald-300'}">
                      {compatibilityReport && !compatibilityReport.is_compatible ? 'Se detectaron posibles incompatibilidades de Mods' : 'Todos los mods son compatibles con esta instancia'}
                    </h3>
                    <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-900 border border-obsidian-750 font-mono text-slate-300">
                      MC {currentInstance.mc_version} • {currentInstance.loader_type}
                    </span>
                  </div>
                  {#if compatibilityReport && compatibilityReport.issues.length > 0}
                    <div class="mt-2 space-y-1.5">
                      {#each compatibilityReport.issues as issue}
                        <div class="text-[11px] p-2 rounded bg-obsidian-950/80 border border-obsidian-800 text-slate-300">
                          <p class="font-bold text-rose-400">⚠️ {issue.title}</p>
                          <p class="text-slate-400 text-[10px] mt-0.5">{issue.description}</p>
                          <p class="text-amber-400 text-[10px] mt-0.5 font-medium">💡 Recomendación: {issue.recommendation}</p>
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <p class="text-[11px] text-slate-400 mt-0.5">
                      Verificación en tiempo real sin conflictos de Modloader o dependencias faltantes conocidas.
                    </p>
                  {/if}
                </div>
              </div>
              <button
                type="button"
                onclick={runCompatibilityCheck}
                disabled={isCheckingCompatibility}
                class="px-2.5 py-1 rounded-lg bg-obsidian-900 border border-obsidian-750 text-xs font-semibold hover:text-white hover:border-omni-indigo transition shrink-0 flex items-center gap-1.5"
              >
                <RotateCw class="w-3.5 h-3.5 {isCheckingCompatibility ? 'animate-spin text-omni-cyan' : 'text-slate-400'}" />
                <span>Re-verificar</span>
              </button>
            </div>
          {/if}

          <!-- Pestañas de Categoría: Mods | Shaders | Recursos + Buscador -->
          <div class="flex items-center justify-between gap-3 bg-obsidian-900/60 p-2.5 rounded-xl border border-obsidian-750/70">
            <div class="flex items-center gap-1.5 bg-obsidian-950 border border-obsidian-800 rounded-lg p-1">
              <button
                type="button"
                onclick={() => { activeContentCategory = 'mods'; loadContentForInstance(currentInstance.folder, 'mods'); }}
                class="px-3 py-1 rounded-md text-xs font-bold transition {activeContentCategory === 'mods' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Mods
              </button>
              <button
                type="button"
                onclick={() => { activeContentCategory = 'shaderpacks'; loadContentForInstance(currentInstance.folder, 'shaderpacks'); }}
                class="px-3 py-1 rounded-md text-xs font-bold transition {activeContentCategory === 'shaderpacks' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Shaders
              </button>
              <button
                type="button"
                onclick={() => { activeContentCategory = 'resourcepacks'; loadContentForInstance(currentInstance.folder, 'resourcepacks'); }}
                class="px-3 py-1 rounded-md text-xs font-bold transition {activeContentCategory === 'resourcepacks' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Resource Packs
              </button>
            </div>

            <div class="relative w-72">
              <Search class="w-3.5 h-3.5 absolute left-3 top-2.5 text-slate-400" />
              <input
                type="text"
                bind:value={contentSearchFilter}
                placeholder={`Filtrar ${activeContentCategory}...`}
                class="w-full bg-obsidian-950 border border-obsidian-750 rounded-lg pl-8 pr-3 py-1.5 text-xs text-white focus:outline-none focus:border-omni-cyan font-mono"
              />
            </div>
          </div>

          <!-- Tabla de Contenido con Interruptores Activar / Desactivar -->
          <div class="rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 overflow-hidden">
            <table class="w-full text-left text-xs">
              <thead class="bg-obsidian-950/80 text-slate-400 border-b border-obsidian-800">
                <tr>
                  <th class="p-3 w-16 text-center">Estado</th>
                  <th class="p-3">Nombre del Archivo / Paquete</th>
                  <th class="p-3">Categoría</th>
                  <th class="p-3 text-right">Tamaño</th>
                  <th class="p-3 w-28 text-center">Acciones</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-obsidian-800">
                {#if filteredContentItems.length === 0}
                  <tr>
                    <td colspan="5" class="p-8 text-center text-slate-500">
                      No hay elementos en esta categoría todavía. Puedes descargarlos desde la pestaña <strong>{t('nav_store')}</strong> o copiarlos directamente en la carpeta.
                    </td>
                  </tr>
                {:else}
                  {#each filteredContentItems as item}
                    <tr class="hover:bg-obsidian-800/40 transition">
                      <td class="p-3 text-center">
                        <button
                          type="button"
                          onclick={() => handleToggleContent(item)}
                          class="transition transform active:scale-95"
                          title={item.enabled ? 'Clic para desactivar' : 'Clic para activar'}
                        >
                          {#if item.enabled}
                            <ToggleRight class="w-6 h-6 text-emerald-400 fill-emerald-400/20" />
                          {:else}
                            <ToggleLeft class="w-6 h-6 text-slate-500 hover:text-slate-400" />
                          {/if}
                        </button>
                      </td>
                      <td class="p-3">
                        <span class="font-semibold {item.enabled ? 'text-white' : 'text-slate-500 line-through'}">
                          {item.name}
                        </span>
                        {#if !item.enabled}
                          <span class="ml-2 px-1.5 py-0.2 rounded text-[9px] bg-red-500/10 text-red-400 border border-red-500/20 font-mono">
                            deshabilitado
                          </span>
                        {/if}
                      </td>
                      <td class="p-3 font-mono text-slate-400 uppercase text-[10px]">
                        {item.category}
                      </td>
                      <td class="p-3 text-right text-slate-400 font-mono">
                        {(item.size_bytes / (1024 * 1024)).toFixed(2)} MB
                      </td>
                      <td class="p-3 text-center flex items-center justify-center gap-1.5">
                        <!-- Botón Escáner Anti-Malware Plus Ultra -->
                        {#if activeContentCategory === 'mods'}
                          <button
                            type="button"
                            onclick={() => handleScanMod(item.filename)}
                            disabled={isScanningMod === item.filename}
                            class="p-1 rounded text-slate-400 hover:text-emerald-400 hover:bg-emerald-500/10 transition"
                            title="Auditoría de seguridad: Escanear en busca de Fracturiser / malware"
                          >
                            {#if isScanningMod === item.filename}
                              <RotateCw class="w-3.5 h-3.5 animate-spin text-omni-cyan" />
                            {:else if scanModResults[item.filename]}
                              {#if scanModResults[item.filename].is_clean}
                                <Shield class="w-3.5 h-3.5 text-emerald-400" />
                              {:else}
                                <AlertTriangle class="w-3.5 h-3.5 text-rose-500 animate-bounce" />
                              {/if}
                            {:else}
                              <Shield class="w-3.5 h-3.5 text-slate-500 hover:text-emerald-400" />
                            {/if}
                          </button>
                        {/if}

                        <button
                          type="button"
                          onclick={() => handleDeleteContent(item)}
                          class="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition"
                          title="Eliminar archivo permanentemente"
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>

      {:else if activeTab === 'store'}
        <!-- PESTAÑA TIENDA EN VIVO CON FILTROS EXPANSIVOS Y CURSEFORGE -->
        <div class="space-y-4 max-w-6xl">
          <!-- Filtros fila 1: Fuente, Búsqueda y Tipo -->
          <div class="flex items-center gap-3 bg-obsidian-900/60 p-3 rounded-xl border border-obsidian-750/70">
            <!-- Selector de Fuente (Multi-Repositorio) -->
            <select
              bind:value={storeSource}
              onchange={doSearchStore}
              class="bg-obsidian-950 border border-obsidian-700 rounded-lg px-3 py-2 text-xs font-bold text-omni-cyan focus:outline-none cursor-pointer"
            >
              <option value="all">🌐 Todas las Fuentes (Modrinth + CurseForge)</option>
              <option value="modrinth">🟢 Modrinth (Oficial Rápido)</option>
              <option value="curseforge">🟠 CurseForge (Eternal API)</option>
            </select>

            <div class="relative flex-1">
              <Search class="w-4 h-4 absolute left-3 top-3 text-slate-400" />
              <input
                type="text"
                bind:value={searchQuery}
                onkeydown={(e) => e.key === 'Enter' && doSearchStore()}
                placeholder="Buscar mods, shaders, resourcepacks o modpacks en catálogo unificado..."
                class="w-full bg-obsidian-950/80 border border-obsidian-700 rounded-lg pl-9 pr-4 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-omni-cyan"
              />
            </div>

            <div class="flex items-center gap-1.5 bg-obsidian-950 border border-obsidian-700 rounded-lg p-1">
              <button
                type="button"
                onclick={() => { storeType = 'mod'; doSearchStore(); }}
                class="px-2.5 py-1 rounded text-xs font-semibold transition {storeType === 'mod' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Mods
              </button>
              <button
                type="button"
                onclick={() => { storeType = 'modpack'; doSearchStore(); }}
                class="px-2.5 py-1 rounded text-xs font-semibold transition {storeType === 'modpack' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Modpacks
              </button>
              <button
                type="button"
                onclick={() => { storeType = 'shader'; doSearchStore(); }}
                class="px-2.5 py-1 rounded text-xs font-semibold transition {storeType === 'shader' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Shaders
              </button>
              <button
                type="button"
                onclick={() => { storeType = 'resourcepack'; doSearchStore(); }}
                class="px-2.5 py-1 rounded text-xs font-semibold transition {storeType === 'resourcepack' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
              >
                Recursos
              </button>
            </div>

            <button
              onclick={doSearchStore}
              disabled={isSearching}
              class="flex items-center gap-2 px-4 py-2 rounded-lg bg-omni-cyan hover:bg-omni-cyan/80 text-obsidian-950 font-bold text-xs transition disabled:opacity-50"
            >
              {#if isSearching}
                <RotateCw class="w-3.5 h-3.5 animate-spin" />
              {:else}
                <Search class="w-3.5 h-3.5" />
              {/if}
              <span>Buscar</span>
            </button>
          </div>

          <!-- Filtros fila 2: Modloader, Versión de MC y Ordenamiento -->
          <div class="flex items-center gap-4 bg-obsidian-900/40 p-2.5 rounded-xl border border-obsidian-800 text-xs">
            <div class="flex items-center gap-2">
              <Filter class="w-3.5 h-3.5 text-omni-cyan" />
              <span class="font-bold text-slate-400">Modloader:</span>
              <select
                bind:value={storeLoaderFilter}
                onchange={doSearchStore}
                class="bg-obsidian-950 border border-obsidian-700 rounded px-2 py-1 text-xs text-white"
              >
                <option value="Auto">Auto ({currentInstance.loader_type})</option>
                <option value="Fabric">Fabric</option>
                <option value="NeoForge">NeoForge</option>
                <option value="Forge">Forge</option>
                <option value="Quilt">Quilt</option>
                <option value="Todos">Todos</option>
              </select>
            </div>

            <div class="flex items-center gap-2">
              <span class="font-bold text-slate-400">Versión MC:</span>
              <select
                bind:value={storeMcFilter}
                onchange={doSearchStore}
                class="bg-obsidian-950 border border-obsidian-700 rounded px-2 py-1 text-xs text-white"
              >
                <option value="Auto">Auto ({currentInstance.mc_version})</option>
                <option value="Todas">Todas</option>
                <option value="1.21.4">1.21.4</option>
                <option value="1.21.1">1.21.1</option>
                <option value="1.20.4">1.20.4</option>
                <option value="1.20.1">1.20.1</option>
                <option value="1.19.2">1.19.2</option>
                <option value="1.18.2">1.18.2</option>
                <option value="1.16.5">1.16.5</option>
                <option value="1.12.2">1.12.2</option>
              </select>
            </div>
          </div>

          <!-- Resultados de Búsqueda -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {#each searchResults as item}
              <div class="p-4 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 hover:border-omni-cyan/50 transition flex flex-col justify-between">
                <div>
                  <div class="flex items-start gap-3">
                    {#if item.icon_url}
                      <img src={item.icon_url} alt="" class="w-12 h-12 rounded-lg object-cover bg-obsidian-950" />
                    {:else}
                      <div class="w-12 h-12 rounded-lg bg-obsidian-950 flex items-center justify-center text-xl">📦</div>
                    {/if}
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between">
                        <h3 class="text-sm font-bold text-white truncate">{item.title}</h3>
                        <div class="flex items-center gap-1.5 ml-2">
                          {#if isItemInstalled(item)}
                            <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-teal-500/20 text-teal-300 border border-teal-500/30 flex items-center gap-1">
                              <Check class="w-2.5 h-2.5" />
                              Instalado
                            </span>
                          {/if}
                          <span class="px-1.5 py-0.5 rounded text-[9px] font-bold {item.source === 'CurseForge' ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'}">
                            {item.source || 'Modrinth'}
                          </span>
                          <span class="text-[10px] text-slate-400 font-mono">{item.downloads.toLocaleString()} dl</span>
                        </div>
                      </div>
                      <p class="text-[11px] text-slate-400 font-medium">Por {item.author}</p>
                    </div>
                  </div>
                  <p class="text-xs text-slate-300 mt-2.5 line-clamp-2">{item.description}</p>
                </div>

                <div class="mt-4 pt-3 border-t border-obsidian-800 flex items-center justify-between">
                  <div class="flex gap-1.5 flex-wrap">
                    {#each (item.loaders || []).slice(0, 3) as ldr}
                      <span class="px-1.5 py-0.5 rounded text-[9px] bg-obsidian-800 text-slate-300 font-mono">{ldr}</span>
                    {/each}
                  </div>
                  <div class="flex items-center gap-2">
                    {#if item.web_url}
                      <a
                        href={item.web_url}
                        target="_blank"
                        class="flex items-center gap-1 text-xs text-slate-400 hover:text-white font-medium"
                      >
                        <span>Web</span>
                        <ExternalLink class="w-3 h-3" />
                      </a>
                    {/if}
                    <button
                      onclick={() => handleInstallMod(item)}
                      disabled={downloadingItems.has(item.id)}
                      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-white font-bold text-xs shadow-md transition disabled:opacity-50 {isItemInstalled(item) ? 'bg-teal-600/80 hover:bg-teal-600 shadow-teal-600/20' : 'bg-omni-indigo hover:bg-omni-indigo/80 shadow-omni-indigo/20'}"
                    >
                      {#if downloadingItems.has(item.id)}
                        <RotateCw class="w-3 h-3 animate-spin" />
                        <span>Instalando...</span>
                      {:else if isItemInstalled(item)}
                        <Check class="w-3 h-3 text-teal-200" />
                        <span>Descargado</span>
                      {:else}
                        <Download class="w-3 h-3" />
                        <span>Instalar</span>
                      {/if}
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>

      {:else if activeTab === 'worlds'}
        <!-- PESTAÑA GESTOR DE MUNDOS Y BACKUPS (CREADORES DE CONTENIDO) -->
        <div class="space-y-6 max-w-6xl">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-bold text-white flex items-center gap-2">
                <Globe class="w-5 h-5 text-omni-cyan" />
                Mundos de Juego & Copias de Seguridad (Saves)
              </h2>
              <p class="text-xs text-slate-400 mt-0.5">
                Crea copias de seguridad de tus partidas antes de grabar episodios, instalar mods o hacer cambios importantes.
              </p>
            </div>
            <div class="flex items-center gap-2">
              <button
                onclick={() => loadWorldsForInstance(currentInstance.folder)}
                class="px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-slate-300 text-xs font-bold transition flex items-center gap-1.5 border border-obsidian-700"
              >
                <RotateCw class="w-3.5 h-3.5" />
                <span>Actualizar</span>
              </button>
              <button
                onclick={() => handleOpenFolder(currentInstance.folder + '/.minecraft/saves')}
                class="px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-amber-400 text-xs font-bold transition flex items-center gap-1.5 border border-obsidian-700"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                <span>Abrir Carpeta Saves</span>
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            {#if instanceWorlds.length === 0}
              <div class="md:col-span-3 p-12 text-center bg-obsidian-900/40 rounded-2xl border border-obsidian-800">
                <Globe class="w-12 h-12 text-slate-600 mx-auto mb-3" />
                <h3 class="text-sm font-bold text-white">No hay mundos guardados en esta instancia</h3>
                <p class="text-xs text-slate-500 mt-1 max-w-md mx-auto">
                  Inicia la instancia y crea un mundo en partida individual. Aparecerá automáticamente aquí para hacer respaldos.
                </p>
              </div>
            {:else}
              {#each instanceWorlds as world}
                <div class="p-4 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 hover:border-omni-cyan/50 transition flex flex-col justify-between">
                  <div class="flex items-start gap-3">
                    {#if world.icon_base64}
                      <img src={world.icon_base64} alt="" class="w-12 h-12 rounded-lg object-cover bg-obsidian-950 border border-obsidian-700" />
                    {:else}
                      <div class="w-12 h-12 rounded-lg bg-emerald-950/40 border border-emerald-500/20 flex items-center justify-center text-xl">
                        🌍
                      </div>
                    {/if}
                    <div class="flex-1 min-w-0">
                      <h4 class="text-sm font-bold text-white truncate">{world.display_name}</h4>
                      <p class="text-[11px] text-slate-400 font-mono mt-0.5">{(world.size_bytes / (1024 * 1024)).toFixed(1)} MB</p>
                      <span class="text-[10px] text-slate-500 block mt-0.5">Modificado: {world.last_modified}</span>
                    </div>
                  </div>

                  <div class="mt-4 pt-3 border-t border-obsidian-800 space-y-2">
                    <div class="flex items-center justify-between">
                      <span class="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-400 font-mono">
                        {world.game_mode || 'Supervivencia'}
                      </span>
                      <button
                        onclick={() => handleBackupWorld(world.folder_name)}
                        disabled={isBackingUp}
                        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold text-xs shadow-md shadow-omni-indigo/20 transition disabled:opacity-50"
                      >
                        <Archive class="w-3.5 h-3.5 text-omni-cyan" />
                        <span>Crear Backup</span>
                      </button>
                    </div>

                    <!-- Lista de Backups Restaurables (Modo Básico / Avanzado) -->
                    {#if world.backups && world.backups.length > 0}
                      <div class="mt-2 pt-2 border-t border-obsidian-850">
                        <span class="text-[10px] font-bold text-slate-400 block mb-1">Copias Disponibles ({world.backups.length}):</span>
                        <div class="space-y-1 max-h-24 overflow-y-auto pr-1">
                          {#each world.backups as bkp}
                            <div class="flex items-center justify-between bg-obsidian-950/70 p-1.5 rounded border border-obsidian-800 text-[10px]">
                              <span class="text-slate-300 font-mono truncate max-w-[140px]" title={bkp}>{bkp}</span>
                              <button
                                onclick={() => handleRestoreWorld(bkp)}
                                class="px-2 py-0.5 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 font-bold transition flex items-center gap-1 border border-amber-500/40"
                                title="Restaurar este estado"
                              >
                                <History class="w-3 h-3" /> Restaurar
                              </button>
                            </div>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

      {:else if activeTab === 'logs'}
        <!-- PESTAÑA CONSOLA & LOGS EN VIVO -->
        <div class="space-y-4 max-w-6xl h-full flex flex-col">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-bold text-white flex items-center gap-2">
                <Terminal class="w-5 h-5 text-emerald-400" />
                Consola y Registro de Ejecución (latest.log)
              </h2>
              <p class="text-xs text-slate-400 mt-0.5">
                Monitorea en tiempo real errores, advertencias de mods y el arranque del motor del juego.
              </p>
            </div>
            <div class="flex items-center gap-2 text-xs">
              <span class="text-slate-400 font-mono text-[11px]">
                {instanceLogs.total_lines} líneas • {instanceLogs.last_updated}
              </span>
              <button
                onclick={() => isAutoRefreshLogs = !isAutoRefreshLogs}
                class="px-3 py-1.5 rounded-lg border font-bold transition flex items-center gap-1.5 {isAutoRefreshLogs ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30' : 'bg-obsidian-800 text-slate-300 border-obsidian-700'}"
              >
                <RotateCw class="w-3.5 h-3.5 {isAutoRefreshLogs ? 'animate-spin' : ''}" />
                <span>{isAutoRefreshLogs ? 'Auto-refresco Activo' : 'Pausado'}</span>
              </button>
              <button
                onclick={() => loadLogsForInstance(currentInstance.folder)}
                class="px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-white font-bold transition border border-obsidian-700"
              >
                Refrescar
              </button>
            </div>
          </div>

          <!-- Visor de Consola Terminal Negro -->
          <div class="flex-1 min-h-[460px] bg-black/90 border border-obsidian-750/90 rounded-2xl p-4 font-mono text-[11px] overflow-y-auto space-y-1 shadow-inner select-text">
            {#if instanceLogs.lines.length === 0}
              <div class="text-slate-500 italic">No se han registrado eventos todavía. Inicia el juego para ver los logs en vivo.</div>
            {:else}
              {#each instanceLogs.lines as line}
                <div class="leading-relaxed {line.includes('ERROR') || line.includes('FATAL') ? 'text-rose-400 font-bold bg-rose-500/10 px-1 rounded' : (line.includes('WARN') ? 'text-amber-300' : 'text-slate-300')}">
                  {line}
                </div>
              {/each}
            {/if}
          </div>

          <!-- Input Interactivo de Comandos para Servidores Dedicados -->
          {#if currentInstance.instance_type === 'server'}
            <form onsubmit={(e) => { e.preventDefault(); handleSendServerCommand(); }} class="flex items-center gap-2 pt-1">
              <span class="text-emerald-400 font-mono font-bold text-xs select-none">&gt;</span>
              <input
                type="text"
                bind:value={serverInputCmd}
                placeholder="Enviar comando al servidor (ej. /op jugador, /whitelist on, /say Hola, /stop)..."
                class="flex-1 bg-black/80 border border-obsidian-750 rounded-xl px-3.5 py-2 text-xs text-white focus:outline-none focus:border-emerald-400 font-mono placeholder:text-slate-600"
              />
              <button
                type="submit"
                class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs rounded-xl shadow transition"
              >
                Enviar
              </button>
            </form>
          {/if}
        </div>

      {:else if activeTab === 'config'}
        <!-- PESTAÑA AJUSTES, API KEYS, IDIOMA Y JAVA -->
        <div class="space-y-6 max-w-6xl">
          <div>
            <h2 class="text-base font-bold text-white">{t('config_title')}</h2>
            <p class="text-xs text-slate-400">{t('config_desc')}</p>
          </div>

          <!-- Card de Selección de Idioma del Sistema (i18n Internacionalización) -->
          <div class="p-5 rounded-2xl bg-obsidian-900/60 border border-obsidian-750/80 space-y-3">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-sm font-bold text-white flex items-center gap-2">
                  <Languages class="w-4 h-4 text-omni-indigo" />
                  {t('language_label')}
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  {t('language_expandable_hint')}
                </p>
              </div>
            </div>

            <div class="flex flex-wrap items-center gap-2.5 pt-1">
              {#each availableLanguages as lang}
                <button
                  onclick={() => handleLanguageChange(lang.code)}
                  class="flex items-center gap-2 px-3.5 py-2 rounded-xl border text-xs font-bold transition shadow-sm {currentLang === lang.code ? 'bg-omni-indigo text-white border-indigo-500 shadow-indigo-500/20' : 'bg-obsidian-950 text-slate-300 border-obsidian-750 hover:bg-obsidian-800 hover:text-white'}"
                >
                  <span class="text-base">{lang.flag}</span>
                  <span>{lang.name}</span>
                  {#if currentLang === lang.code}
                    <Check class="w-3.5 h-3.5 text-emerald-400" />
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- Card de Autenticación de Repositorios (CurseForge / Modrinth) -->
          <div class="p-5 rounded-2xl bg-obsidian-900/60 border border-obsidian-750/80 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-sm font-bold text-white flex items-center gap-2">
                  <Shield class="w-4 h-4 text-amber-400" />
                  Autenticación de Tiendas de Mods (CurseForge Eternal / Modrinth)
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Omnicraft incluye por defecto una clave de API pública comunitaria para CurseForge. Si dispones de tu propia clave personal o token privado, puedes configurarla aquí.
                </p>
              </div>
              {#if isApiKeySaved}
                <span class="px-2.5 py-1 rounded-lg bg-emerald-500/20 text-emerald-400 font-bold text-xs flex items-center gap-1 border border-emerald-500/30">
                  <Check class="w-3.5 h-3.5" /> Clave Guardada en SQLite
                </span>
              {/if}
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
              <div class="space-y-1.5">
                <span class="text-xs font-bold text-slate-300">CurseForge API Key ($2a$10$... o Personal):</span>
                <div class="flex gap-2">
                  <input
                    type="password"
                    bind:value={curseforgeApiKey}
                    placeholder="Dejar en blanco para usar la clave comunitaria integrada"
                    class="flex-1 bg-obsidian-950 border border-obsidian-700 rounded-lg px-3 py-2 text-xs text-white focus:outline-none focus:border-amber-400 font-mono"
                  />
                  <button
                    onclick={() => saveApiKey('curseforge', curseforgeApiKey)}
                    class="px-4 py-2 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 text-amber-400 font-bold text-xs border border-amber-500/30 transition"
                  >
                    Guardar
                  </button>
                </div>
              </div>

              <div class="space-y-1.5">
                <span class="text-xs font-bold text-slate-300">Modrinth Personal Access Token (Opcional):</span>
                <div class="flex gap-2">
                  <input
                    type="password"
                    bind:value={modrinthToken}
                    placeholder="Token de acceso opcional para límites ampliados"
                    class="flex-1 bg-obsidian-950 border border-obsidian-700 rounded-lg px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-400 font-mono"
                  />
                  <button
                    onclick={() => saveApiKey('modrinth', modrinthToken)}
                    class="px-4 py-2 rounded-lg bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 font-bold text-xs border border-emerald-500/30 transition"
                  >
                    Guardar
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-bold text-omni-indigo uppercase tracking-wider">
                Entornos Java Detectados en el Sistema ({javas.length})
              </h3>
              <div class="flex items-center gap-1.5">
                <span class="text-[11px] text-slate-400">Instalar OpenJDK:</span>
                <button
                  onclick={() => handleDownloadTemurinJre(21)}
                  disabled={isDownloadingJre !== null}
                  class="px-2.5 py-1 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 border border-obsidian-700 text-omni-cyan font-bold text-xs transition disabled:opacity-50"
                  title="Java 21 (Minecraft 1.20.5+)"
                >
                  {isDownloadingJre === 21 ? 'Descargando...' : '+ Java 21 LTS'}
                </button>
                <button
                  onclick={() => handleDownloadTemurinJre(17)}
                  disabled={isDownloadingJre !== null}
                  class="px-2.5 py-1 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 border border-obsidian-700 text-emerald-400 font-bold text-xs transition disabled:opacity-50"
                  title="Java 17 (Minecraft 1.18 a 1.20.4)"
                >
                  {isDownloadingJre === 17 ? 'Descargando...' : '+ Java 17 LTS'}
                </button>
                <button
                  onclick={() => handleDownloadTemurinJre(8)}
                  disabled={isDownloadingJre !== null}
                  class="px-2.5 py-1 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 border border-obsidian-700 text-amber-400 font-bold text-xs transition disabled:opacity-50"
                  title="Java 8 (Minecraft clásico 1.16.5 y anteriores)"
                >
                  {isDownloadingJre === 8 ? 'Descargando...' : '+ Java 8'}
                </button>
              </div>
            </div>

            <div class="space-y-2">
              {#each javas as jv}
                <div class="p-3.5 rounded-xl bg-obsidian-900/60 border border-obsidian-750/70 flex items-center justify-between text-xs">
                  <div class="flex items-center gap-3">
                    <Coffee class="w-5 h-5 text-amber-400" />
                    <div>
                      <span class="font-bold text-white text-sm">Java {jv.version}</span>
                      <span class="text-slate-400 ml-2 font-mono">{jv.vendor} ({jv.is_64bit ? '64-bit' : '32-bit'})</span>
                      <p class="text-[11px] text-slate-400 font-mono mt-0.5">{jv.path}</p>
                    </div>
                  </div>
                  <span class="px-2.5 py-1 rounded-full bg-emerald-500/10 text-emerald-400 font-mono text-[10px] border border-emerald-500/20">
                    Compatible
                  </span>
                </div>
              {/each}
            </div>
          </div>

          <!-- SECCIÓN DE INFORMACIÓN, CRÉDITOS, LICENCIA Y DONACIONES AL CREADOR -->
          <div class="p-5 rounded-2xl bg-gradient-to-br from-obsidian-900/90 via-obsidian-900/50 to-pink-950/20 border border-pink-500/20 space-y-4">
            <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <Heart class="w-5 h-5 text-pink-500 fill-pink-500 animate-pulse" />
                  <h3 class="text-sm font-bold text-white">Omnicraft Studio v1.0 — Suite Todo-Terreno de Autoría Propia</h3>
                </div>
                <p class="text-xs text-slate-400 max-w-2xl leading-relaxed">
                  Desarrollado de forma libre y gratuita para todos los sistemas operativos, combinando lo mejor de las herramientas más destacadas de la comunidad de Minecraft en una solución unificada, ágil y de alto rendimiento.
                </p>
              </div>

              <button
                type="button"
                onclick={() => openExternalUrl('https://github.com/correakarl/omnicraft-studio')}
                class="px-4 py-2.5 rounded-xl bg-gradient-to-r from-pink-600 to-rose-600 hover:from-pink-500 hover:to-rose-500 text-white font-bold text-xs shadow-lg shadow-pink-600/25 flex items-center gap-2 transition transform active:scale-95 shrink-0"
              >
                <Heart class="w-4 h-4 fill-white" />
                <span>Apoyar en GitHub / PayPal</span>
                <ExternalLink class="w-3.5 h-3.5 opacity-80" />
              </button>
            </div>

            <!-- Reconocimiento y Agradecimiento Formal -->
            <div class="p-3.5 rounded-xl bg-obsidian-950/70 border border-obsidian-800 text-xs text-slate-300 space-y-1.5">
              <span class="text-amber-400 font-bold block text-[11px] uppercase tracking-wider">Agradecimiento Formal e Inspiración:</span>
              <p class="text-[11px] text-slate-400 leading-relaxed">
                Agradecemos profundamente el trabajo e ingenio pionero de <strong class="text-slate-200">HeberonYT (MultiMinecraft 2 Deluxe)</strong> por el concepto de aislamiento atómico de carpetas, a <strong class="text-slate-200">Prism Launcher / MultiMC</strong> por los estándares abiertos, y a las plataformas <strong class="text-slate-200">Modrinth & CurseForge</strong> por sus APIs. Sus aportes nos motivaron a crear esta versión todo-terreno optimizada desde cero.
              </p>
            </div>

            <div class="pt-3 border-t border-obsidian-800/80 flex flex-wrap items-center justify-between text-xs text-slate-400 gap-3">
              <div class="flex items-center gap-2">
                <span class="text-slate-500">Donaciones Seguras:</span>
                <button
                  type="button"
                  onclick={() => openExternalUrl('https://www.paypal.com/ncp/payment/KR4E3PNGJEXQY')}
                  class="px-2.5 py-1 rounded bg-obsidian-950 border border-pink-500/30 text-pink-400 font-medium hover:bg-pink-950/30 flex items-center gap-1.5 transition"
                >
                  <Heart class="w-3 h-3 fill-pink-400" />
                  <span>Pasarela Oficial PayPal</span>
                </button>
              </div>
              <div class="flex items-center gap-2 text-slate-400 text-[11px]">
                <span class="px-2 py-0.5 rounded bg-obsidian-800/90 border border-obsidian-700 text-slate-300 font-mono">Licencia CC BY-NC-SA 4.0</span>
                <span>• Autor:</span>
                <span class="font-bold text-slate-200">Karl Correa</span>
                <span class="text-slate-600">•</span>
                <button
                  type="button"
                  onclick={() => openExternalUrl('https://github.com/correakarl/omnicraft-studio')}
                  class="text-omni-indigo hover:underline flex items-center gap-1"
                >
                  <Github class="w-3.5 h-3.5" /> GitHub
                </button>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>

<!-- MODAL CREACIÓN / EDICIÓN DE INSTANCIA -->
<InstanceModal
  isOpen={isInstanceModalOpen}
  mode={instanceModalMode}
  instance={instanceModalMode === 'edit' ? currentInstance : null}
  {availableIcons}
  {launchers}
  {javas}
  gameVersions={allGameVersions}
  {uiMode}
  onClose={() => isInstanceModalOpen = false}
  onSave={handleSaveInstanceModal}
/>

<!-- MODAL GESTIÓN DE CUENTAS & SKINS -->
<AccountModal
  isOpen={isAccountModalOpen}
  {accounts}
  onClose={() => isAccountModalOpen = false}
  onSetAccount={handleSetAccount}
  onDeleteAccount={handleDeleteAccount}
  onLogoutUser={handleLogoutUser}
/>

<!-- MODAL ACTUALIZACIÓN DE OMNICRAFT -->
{#if isUpdateModalOpen && appUpdate}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="bg-obsidian-900 border border-obsidian-750/90 rounded-2xl w-full max-w-lg p-6 flex flex-col shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-obsidian-750 pb-3">
        <div class="flex items-center gap-2">
          <Sparkles class="w-5 h-5 text-omni-cyan" />
          <h3 class="text-sm font-bold text-white">Actualización de Omnicraft Launcher Studio</h3>
        </div>
        <span class="text-xs font-mono px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
          {appUpdate.latest_version}
        </span>
      </div>

      <div class="text-xs text-slate-300 space-y-2 max-h-60 overflow-y-auto bg-obsidian-950 p-3 rounded-xl border border-obsidian-800 font-sans leading-relaxed whitespace-pre-wrap">
        {appUpdate.release_notes}
      </div>

      <div class="flex items-center justify-between pt-2">
        <span class="text-[11px] text-slate-400 font-mono">Versión actual: {appUpdate.current_version}</span>
        <div class="flex gap-2">
          <button
            onclick={() => isUpdateModalOpen = false}
            class="px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-slate-300 font-bold text-xs"
          >
            Cerrar
          </button>
          <a
            href={appUpdate.download_url}
            target="_blank"
            class="px-3 py-1.5 rounded-lg bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold text-xs flex items-center gap-1.5 shadow"
          >
            <Download class="w-3.5 h-3.5" /> Descargar Actualización
          </a>
        </div>
      </div>
    </div>
  </div>
{/if}


<!-- MODAL EDITOR DE SKINS (OFFLINE Y MOJANG) -->
<SkinEditorModal
  isOpen={isSkinEditorModalOpen}
  {activeAccount}
  {localSkins}
  onClose={() => isSkinEditorModalOpen = false}
  onSaveSkin={handleSaveLocalSkin}
  onInjectOffline={handleInjectOfflineSkin}
  onUploadMojang={handleUploadMojang}
  onDeleteSkin={handleDeleteLocalSkin}
  onRefreshSkins={handleRefreshLocalSkins}
/>

<!-- GESTOR FLOTANTE DE DESCARGAS Y COLA EN VIVO -->
{#if showDownloadsDrawer}
  <div class="fixed bottom-4 right-4 z-50 w-96 max-w-[calc(100vw-2rem)] bg-obsidian-900 border border-obsidian-750/90 rounded-2xl shadow-2xl overflow-hidden backdrop-blur-md animate-in slide-in-from-bottom duration-200">
    <!-- Header del Gestor -->
    <div class="px-4 py-3 bg-obsidian-950/80 border-b border-obsidian-800 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Download class="w-4 h-4 text-amber-400" />
        <h4 class="text-xs font-bold text-white tracking-wide uppercase">Cola de Descargas & Modpacks</h4>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] px-2 py-0.5 rounded-full bg-obsidian-900 border border-obsidian-750 text-slate-300 font-mono">
          {downloadTasks.length} {downloadTasks.length === 1 ? 'tarea' : 'tareas'}
        </span>
        <button
          onclick={() => showDownloadsDrawer = false}
          class="p-1 rounded text-slate-400 hover:text-white hover:bg-obsidian-800 transition"
          title="Cerrar panel de descargas"
        >
          <XCircle class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Lista de Descargas -->
    <div class="p-3 space-y-2.5 max-h-80 overflow-y-auto divide-y divide-obsidian-800/60">
      {#if downloadTasks.length === 0}
        <div class="text-center py-6 text-slate-500 text-xs">
          No hay descargas activas en este momento.
        </div>
      {:else}
        {#each downloadTasks as task}
          <div class="pt-2.5 first:pt-0 space-y-1.5">
            <div class="flex items-start justify-between gap-2">
              <div class="min-w-0">
                <p class="text-xs font-bold text-white truncate">{task.title}</p>
                <div class="flex items-center gap-2 mt-0.5 text-[10px] text-slate-400 font-mono">
                  <span>{task.source}</span>
                  <span>•</span>
                  <span class="{task.completed ? 'text-emerald-400 font-bold' : 'text-amber-400'}">{task.status}</span>
                </div>
              </div>
              <div class="text-right shrink-0">
                {#if task.completed}
                  <CheckCircle2 class="w-4 h-4 text-emerald-400" />
                {:else}
                  <RotateCw class="w-4 h-4 text-amber-400 animate-spin" />
                {/if}
              </div>
            </div>

            <!-- Barra de Progreso -->
            <div class="w-full bg-obsidian-950 h-2 rounded-full overflow-hidden border border-obsidian-800">
              <div
                class="h-full transition-all duration-200 {task.completed ? 'bg-emerald-500' : 'bg-gradient-to-r from-omni-indigo to-amber-500'}"
                style="width: {Math.max(5, Math.min(100, task.progress))}%"
              ></div>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- Footer del Gestor -->
    <div class="px-4 py-2 bg-obsidian-950/60 border-t border-obsidian-800/80 flex items-center justify-between text-[10px] text-slate-400">
      <span>Los archivos se instalan en <strong class="text-slate-300">.minecraft/</strong></span>
      {#if downloadTasks.some(t => t.completed)}
        <button
          onclick={() => downloadTasks = downloadTasks.filter(t => !t.completed)}
          class="text-omni-cyan hover:underline font-semibold"
        >
          Limpiar completadas
        </button>
      {/if}
    </div>
  </div>
{/if}

<!-- MODAL REPOSITORIO COMPARTIDO (CERO ESPACIO DUPLICADO) -->
{#if showSharedCacheModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="bg-obsidian-900 border border-obsidian-750/90 rounded-2xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-obsidian-750 flex items-center justify-between bg-obsidian-900/90">
        <div>
          <h2 class="text-sm font-bold text-white flex items-center gap-2">
            <Layers class="w-4 h-4 text-teal-400" />
            Repositorio de Contenido Compartido
          </h2>
          <p class="text-xs text-slate-400 mt-0.5">
            Vincula contenido ya descargado en otras instancias sin ocupar espacio extra en disco (Hardlinks nativos).
          </p>
        </div>
        <button
          onclick={() => showSharedCacheModal = false}
          class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-obsidian-800 transition"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Body Lista de Contenido en Caché Compartida -->
      <div class="p-6 overflow-y-auto space-y-3 text-xs flex-1">
        <div class="flex items-center justify-between pb-2 border-b border-obsidian-800 text-[11px] text-slate-400">
          <span>Categoría actual: <strong class="text-omni-cyan uppercase">{activeContentCategory}</strong></span>
          <span>Instancia de destino: <strong class="text-white">{currentInstance.name}</strong></span>
        </div>

        {#if sharedCacheItems.length === 0}
          <div class="p-8 text-center text-slate-500 border border-dashed border-obsidian-800 rounded-xl space-y-1.5">
            <Package class="w-8 h-8 mx-auto opacity-40 text-slate-400" />
            <p class="font-semibold text-slate-400">No hay contenido en la caché central para esta categoría</p>
            <p class="text-[11px]">Cuando descargues mods, shaders o recursos en cualquier instancia, se guardarán aquí automáticamente.</p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each sharedCacheItems as sItem}
              <div class="p-3 rounded-xl bg-obsidian-950/70 border border-obsidian-800 flex items-center justify-between hover:border-obsidian-700 transition">
                <div class="min-w-0 flex-1 pr-3">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-xs font-bold text-white truncate">{sItem.filename}</span>
                    <span class="text-[10px] font-mono text-slate-500">{(sItem.size_bytes / (1024 * 1024)).toFixed(2)} MB</span>
                  </div>
                  <p class="text-[10px] text-slate-400 mt-0.5">
                    {#if sItem.is_linked_to_instance}
                      <span class="text-emerald-400 font-semibold">✓ Ya activo en esta instancia</span>
                    {:else}
                      <span class="text-slate-500">Disponible para vincular en 1 clic (0 MB en disco)</span>
                    {/if}
                  </p>
                </div>

                <div>
                  {#if sItem.is_linked_to_instance}
                    <span class="px-2.5 py-1 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[11px] font-bold">
                      Vinculado
                    </span>
                  {:else}
                    <button
                      onclick={() => handleLinkSharedItem(sItem.filename)}
                      disabled={isLinkingShared}
                      class="px-3 py-1.5 rounded-lg bg-teal-600 hover:bg-teal-500 text-white font-bold text-xs shadow-md shadow-teal-600/20 transition flex items-center gap-1.5 disabled:opacity-50"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      <span>Vincular (0 MB)</span>
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-6 py-3 border-t border-obsidian-800 bg-obsidian-950 flex items-center justify-between text-[11px] text-slate-400">
        <span>Los enlaces no consumen espacio adicional en tu SSD/HDD.</span>
        <button
          onclick={() => showSharedCacheModal = false}
          class="px-4 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-700 text-white font-semibold transition"
        >
          Cerrar
        </button>
      </div>
    </div>
  </div>
{/if}

