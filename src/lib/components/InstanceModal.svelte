<script lang="ts">
  import { X, Check, FolderOpen, Image as ImageIcon, Cpu, Terminal, Shield, Filter } from 'lucide-svelte';
  import type { InstanceModel, JavaRuntime, GameVersionEntry } from '../types';

  let {
    isOpen = false,
    mode = 'create',
    instance = null as InstanceModel | null,
    availableIcons = [] as string[],
    launchers = [] as string[],
    javas = [] as JavaRuntime[],
    gameVersions = [] as GameVersionEntry[],
    uiMode = 'basic' as 'basic' | 'advanced',
    onClose = () => {},
    onSave = (data: any) => {},
  } = $props<{
    isOpen: boolean;
    mode: 'create' | 'edit';
    instance?: InstanceModel | null;
    availableIcons: string[];
    launchers: string[];
    javas: JavaRuntime[];
    gameVersions: GameVersionEntry[];
    uiMode?: 'basic' | 'advanced';
    onClose: () => void;
    onSave: (data: any) => void;
  }>();

  // Form states
  let name = $state('');
  let instanceType = $state<'client' | 'server'>('client');
  let mcVersion = $state('1.21.4');
  let showSnapshots = $state(false);
  let showHistorical = $state(false);

  // Bandera de habilitación de Modloader (si está deshabilitado = Vanilla puro sin enganches)
  let enableModloader = $state(true);
  let loaderType = $state<'Fabric' | 'NeoForge' | 'Forge' | 'Quilt'>('Fabric');
  let loaderBranch = $state<'stable' | 'beta'>('stable'); // Bandera para loaders estables vs experimentales/beta

  let icon = $state('art-Crafting_Table.png');
  let executableType = $state('direct');
  let launcherJar = $state('Motor Nativo Directo (Recomendado - Sin launchers externos)');
  let customExec = $state('');
  let javaMode = $state('auto');
  let customJava = $state('');
  let ramMin = $state(1024);
  let ramMax = $state(4096);
  let jvmPreset = $state('aikar');
  let jvmArgs = $state('');
  let serverPort = $state(25565);
  let assignedUsername = $state('');
  let showIconPicker = $state(false);

  // Detector de versión objetivo de Java
  let targetJavaVersion = $derived.by(() => {
    const v = mcVersion.trim();
    if (v.startsWith('1.21') || v.startsWith('1.20.5') || v.startsWith('1.20.6')) return 21;
    if (v.startsWith('1.18') || v.startsWith('1.19') || v.startsWith('1.20')) return 17;
    if (v.startsWith('1.17')) return 16;
    return 8;
  });

  // Recomendar parámetros automáticamente al cambiar mcVersion en modo create
  function autoTuneForVersion(v: string) {
    if (v.startsWith('1.21') || v.startsWith('1.20.5') || v.startsWith('1.20.6')) {
      if (mode === 'create') {
        jvmPreset = 'generational_zgc';
        ramMin = Math.max(ramMin, 2048);
        ramMax = Math.max(ramMax, 4096);
      }
    } else if (v.startsWith('1.18') || v.startsWith('1.19') || v.startsWith('1.20') || v.startsWith('1.17')) {
      if (mode === 'create') {
        jvmPreset = 'aikar';
        ramMin = Math.max(ramMin, 1024);
        ramMax = Math.max(ramMax, 4096);
      }
    } else {
      // 1.16.5 y anteriores (Java 8)
      if (mode === 'create') {
        jvmPreset = 'aikar';
        if (ramMax > 6144) ramMax = 4096;
      }
    }
  }

  let filteredVersions = $derived.by(() => {
    if (!gameVersions || gameVersions.length === 0) {
      return [
        { id: '1.21.4', version_type: 'release' },
        { id: '1.21.1', version_type: 'release' },
        { id: '1.20.4', version_type: 'release' },
        { id: '1.20.1', version_type: 'release' },
        { id: '1.19.2', version_type: 'release' },
        { id: '1.18.2', version_type: 'release' },
        { id: '1.16.5', version_type: 'release' },
        { id: '1.12.2', version_type: 'release' },
        { id: '1.8.9', version_type: 'release' },
        { id: '1.7.10', version_type: 'release' },
      ];
    }
    return gameVersions.filter(v => {
      if (v.version_type === 'release') return true;
      if (showSnapshots && v.version_type === 'snapshot') return true;
      if (showHistorical && (v.version_type === 'old_beta' || v.version_type === 'old_alpha')) return true;
      return false;
    });
  });

  $effect(() => {
    if (instance && mode === 'edit') {
      name = instance.name;
      instanceType = (instance.instance_type as 'client' | 'server') || 'client';
      mcVersion = instance.mc_version;
      enableModloader = instance.loader_type.toLowerCase() !== 'vanilla';
      if (enableModloader) {
        loaderType = (instance.loader_type as any) || 'Fabric';
      }
      icon = instance.icon || 'art-Crafting_Table.png';
      executableType = instance.executable_type || 'direct';
      launcherJar = instance.launcher_jar || (launchers[0] || 'Motor Nativo Directo (Recomendado - Sin launchers externos)');
      customExec = instance.custom_executable_path || '';
      javaMode = instance.java_selection_mode || 'auto';
      customJava = instance.custom_java_path || '';
      ramMin = instance.ram_min_mb || 1024;
      ramMax = instance.ram_max_mb || 4096;
      jvmPreset = instance.jvm_preset || 'aikar';
      jvmArgs = instance.jvm_args || '';
      serverPort = instance.server_port || 25565;
      assignedUsername = instance.assigned_username || '';
    } else if (mode === 'create') {
      name = '';
      instanceType = 'client';
      mcVersion = '1.21.4';
      enableModloader = true;
      loaderType = 'Fabric';
      icon = 'art-Crafting_Table.png';
      executableType = 'direct';
      launcherJar = launchers[0] || 'Motor Nativo Directo (Recomendado - Sin launchers externos)';
      customExec = '';
      javaMode = 'auto';
      customJava = '';
      ramMin = 1024;
      ramMax = 4096;
      jvmPreset = 'aikar';
      jvmArgs = '';
      serverPort = 25565;
      assignedUsername = '';
    }
  });

  function handleSubmit() {
    if (!name.trim()) return;
    const finalLoader = enableModloader ? loaderType : 'Vanilla';
    onSave({
      id: instance?.id || '',
      name: name.trim(),
      instance_type: instanceType,
      mc_version: mcVersion,
      loader_type: finalLoader,
      icon,
      executable_type: executableType,
      launcher_jar: executableType === 'jar_bundle' ? launcherJar : null,
      custom_exec: executableType === 'custom_jar' ? customExec : null,
      java_mode: javaMode,
      custom_java: javaMode === 'custom' ? customJava : null,
      ram_min: Number(ramMin),
      ram_max: Number(ramMax),
      jvm_preset: jvmPreset,
      jvm_args: jvmArgs,
      server_port: Number(serverPort),
      assigned_username: assignedUsername.trim() || null,
    });
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="bg-obsidian-900 border border-obsidian-750/90 rounded-2xl w-full max-w-2xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-obsidian-750 flex items-center justify-between bg-obsidian-900/90">
        <div>
          <h2 class="text-base font-bold text-white flex items-center gap-2">
            {#if mode === 'create'}
              <span class="w-6 h-6 rounded-lg bg-emerald-500/20 text-emerald-400 flex items-center justify-center text-xs">＋</span>
              Nueva Instancia de Minecraft
            {:else}
              <span class="w-6 h-6 rounded-lg bg-omni-indigo/20 text-omni-lavender flex items-center justify-center text-xs">✏️</span>
              Configurar Instancia: {instance?.name}
            {/if}
          </h2>
          <p class="text-xs text-slate-400 mt-0.5">
            Personaliza el aislamiento de carpetas, versión, modloader, ejecutable y Java.
          </p>
        </div>
        <button onclick={onClose} class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-obsidian-800 transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Scrollable Form Body -->
      <div class="p-6 overflow-y-auto space-y-5 text-xs text-slate-200">
        <!-- Propósito / Tipo -->
        <div class="grid grid-cols-2 gap-3">
          <button
            type="button"
            onclick={() => instanceType = 'client'}
            class="p-3.5 rounded-xl border text-left transition flex items-center gap-3 {instanceType === 'client' ? 'bg-omni-indigo/20 border-omni-indigo text-white shadow-sm' : 'bg-obsidian-800/40 border-obsidian-750 text-slate-400 hover:border-obsidian-600'}"
          >
            <div class="text-xl">🎮</div>
            <div>
              <p class="font-bold text-sm text-white">Instancia Cliente</p>
              <p class="text-[11px] text-slate-400">Para jugar con shaders y mods</p>
            </div>
          </button>

          <button
            type="button"
            onclick={() => instanceType = 'server'}
            class="p-3.5 rounded-xl border text-left transition flex items-center gap-3 {instanceType === 'server' ? 'bg-emerald-500/20 border-emerald-500 text-white shadow-sm' : 'bg-obsidian-800/40 border-obsidian-750 text-slate-400 hover:border-obsidian-600'}"
          >
            <div class="text-xl">🖥️</div>
            <div>
              <p class="font-bold text-sm text-white">Servidor Dedicado</p>
              <p class="text-[11px] text-slate-400">Multiplayer local / LAN / Online</p>
            </div>
          </button>
        </div>

        <!-- Nombre e Icono -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
          <div class="md:col-span-3">
            <label class="block text-slate-300 font-bold mb-1.5">Nombre de la Instancia</label>
            <input
              type="text"
              bind:value={name}
              placeholder="Ej: Survival 1.21 Fabric o Mi Servidor"
              class="w-full bg-obsidian-950 border border-obsidian-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-omni-cyan font-medium"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-bold mb-1.5">Icono Temático</label>
            <button
              type="button"
              onclick={() => showIconPicker = !showIconPicker}
              class="w-full h-[38px] bg-obsidian-950 border border-obsidian-700 hover:border-omni-cyan rounded-lg px-2 flex items-center justify-between text-slate-300 transition"
            >
              <img src={`/minecraft-icons/${icon}`} alt="icono" class="w-6 h-6 object-contain" />
              <span class="text-[10px] text-slate-400 font-mono truncate max-w-[60px]">{icon.replace('.png', '')}</span>
            </button>
          </div>
        </div>

        <!-- Usuario / Jugador Asignado a esta Instancia -->
        <div class="p-3 bg-obsidian-950/60 border border-obsidian-800 rounded-xl">
          <div class="flex items-center justify-between mb-1.5">
            <label class="text-slate-300 font-bold flex items-center gap-1.5">
              <span>👤</span>
              <span>Nombre de Usuario / Jugador en esta Instancia</span>
            </label>
            <span class="text-[10px] text-slate-500 font-mono">Multi-instancia independiente</span>
          </div>
          <input
            type="text"
            bind:value={assignedUsername}
            placeholder="Ej: Player_Prueba (dejar vacío para heredar cuenta activa)"
            class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none focus:border-omni-cyan font-mono text-xs"
          />
          <p class="text-[10px] text-slate-400 mt-1">
            Permite abrir múltiples instancias en paralelo sin colisión de nombres. Si una instancia usa cuenta Microsoft oficial, las demás deben tener su propio usuario offline asignado.
          </p>
        </div>

        <!-- Selector emergente de iconos si está abierto -->
        {#if showIconPicker}
          <div class="p-3 bg-obsidian-950/90 border border-obsidian-750 rounded-xl max-h-44 overflow-y-auto">
            <div class="grid grid-cols-8 gap-2">
              {#each availableIcons as ic}
                <button
                  type="button"
                  onclick={() => { icon = ic; showIconPicker = false; }}
                  class="p-1.5 rounded-lg border transition hover:scale-110 flex items-center justify-center {icon === ic ? 'border-omni-cyan bg-omni-cyan/15' : 'border-obsidian-800 hover:border-obsidian-600'}"
                  title={ic}
                >
                  <img src={`/minecraft-icons/${ic}`} alt={ic} class="w-7 h-7 object-contain" />
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Versión y Modloader -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label class="block text-slate-300 font-bold">Versión de Minecraft</label>
              <!-- Flags / Checkboxes para habilitar Snapshots y Versiones Históricas -->
              <div class="flex items-center gap-2.5 text-[11px]">
                <label class="flex items-center gap-1 cursor-pointer text-slate-400 hover:text-white select-none">
                  <input
                    type="checkbox"
                    bind:checked={showSnapshots}
                    class="rounded bg-obsidian-950 border-obsidian-700 accent-omni-indigo cursor-pointer w-3.5 h-3.5"
                  />
                  <span>Snapshots</span>
                </label>
                <label class="flex items-center gap-1 cursor-pointer text-slate-400 hover:text-white select-none">
                  <input
                    type="checkbox"
                    bind:checked={showHistorical}
                    class="rounded bg-obsidian-950 border-obsidian-700 accent-amber-500 cursor-pointer w-3.5 h-3.5"
                  />
                  <span>Beta/Alpha</span>
                </label>
              </div>
            </div>
            <select
              bind:value={mcVersion}
              onchange={(e: any) => autoTuneForVersion(e.target.value)}
              class="w-full bg-obsidian-950 border border-obsidian-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-omni-cyan cursor-pointer font-mono text-xs max-h-48"
            >
              {#each filteredVersions as v}
                <option value={v.id}>
                  {v.id} {v.version_type !== 'release' ? `[${v.version_type}]` : ''}
                </option>
              {/each}
            </select>
            <p class="text-[10px] text-slate-500 mt-1">
              {#if showSnapshots && showHistorical}
                Mostrando releases, snapshots y versiones históricas ({filteredVersions.length} versiones).
              {:else if showSnapshots}
                Mostrando releases y snapshots ({filteredVersions.length} versiones).
              {:else if showHistorical}
                Mostrando releases y versiones históricas ({filteredVersions.length} versiones).
              {:else}
                Mostrando solo versiones finales estables ({filteredVersions.length} releases).
              {/if}
            </p>
          </div>

          <!-- Columna Modloader con banderas de habilitación y ramas -->
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label class="block text-slate-300 font-bold">Modloader (Enganche al Juego)</label>
              <!-- Bandera para activar o desactivar modloaders (Vanilla puro) -->
              <div class="flex items-center gap-2 text-[11px]">
                <label class="flex items-center gap-1 cursor-pointer text-slate-400 hover:text-white select-none">
                  <input
                    type="checkbox"
                    bind:checked={enableModloader}
                    class="rounded bg-obsidian-950 border-obsidian-700 accent-emerald-500 cursor-pointer w-3.5 h-3.5"
                  />
                  <span class="{enableModloader ? 'text-emerald-400 font-bold' : 'text-slate-500'}">Enganchar Mods</span>
                </label>
              </div>
            </div>

            {#if enableModloader}
              <select
                bind:value={loaderType}
                class="w-full bg-obsidian-950 border border-obsidian-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-omni-cyan cursor-pointer text-xs"
              >
                <option value="Fabric">Fabric (Recomendado Ultra-Rápido & Shaders)</option>
                <option value="NeoForge">NeoForge (Moderno 1.20.2+)</option>
                <option value="Forge">Minecraft Forge (Ecosistema Clásico)</option>
                <option value="Quilt">Quilt (Modular Compatible con Fabric)</option>
              </select>
              <div class="flex items-center justify-between mt-1 text-[10px]">
                <span class="text-slate-400">Modloader activo: <strong class="text-omni-cyan">{loaderType}</strong></span>
                <span class="px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-mono">
                  Inyectado en Launcher
                </span>
              </div>
            {:else}
              <div class="w-full bg-obsidian-950/60 border border-dashed border-obsidian-750 rounded-lg px-3 py-2 text-slate-400 text-xs flex items-center justify-between">
                <span>Vanilla Puro (Sin Modloader)</span>
                <span class="px-1.5 py-0.5 rounded bg-slate-800 text-slate-300 text-[10px]">Limpio</span>
              </div>
              <p class="text-[10px] text-slate-500 mt-1">
                La instancia arrancará en modo Vanilla original sin cargar APIs de mods.
              </p>
            {/if}
          </div>
        </div>

        <!-- Tipo de Lanzador / Motor Ejecutable (Oculto en Modo Básico, Visible en Modo Avanzado) -->
        {#if uiMode === 'advanced'}
          <div class="p-3.5 bg-obsidian-950/60 border border-obsidian-800 rounded-xl space-y-3">
            <label class="block text-slate-300 font-bold flex items-center gap-2">
              <Terminal class="w-4 h-4 text-omni-cyan" />
              Lanzador / Motor de Ejecución
            </label>
            <div class="grid grid-cols-3 gap-2">
              <button
                type="button"
                onclick={() => { executableType = 'direct'; launcherJar = launchers[0] || 'Motor Nativo Directo'; }}
                class="p-2.5 rounded-lg border text-left transition {executableType === 'direct' ? 'bg-omni-indigo/20 border-omni-indigo text-white font-bold' : 'border-obsidian-750 text-slate-400 hover:border-obsidian-600'}"
              >
                <div>Motor Nativo Directo</div>
                <div class="text-[9px] text-omni-cyan font-normal">Recomendado (Sin intermediarios)</div>
              </button>
              <button
                type="button"
                onclick={() => executableType = 'jar_bundle'}
                class="p-2.5 rounded-lg border text-left transition {executableType === 'jar_bundle' ? 'bg-omni-indigo/20 border-omni-indigo text-white font-bold' : 'border-obsidian-750 text-slate-400 hover:border-obsidian-600'}"
              >
                <div>Lanzador Externo (.JAR)</div>
                <div class="text-[9px] text-slate-500 font-normal">HMCL / SKlauncher / Fenix</div>
              </button>
              <button
                type="button"
                onclick={() => executableType = 'custom_jar'}
                class="p-2.5 rounded-lg border text-left transition {executableType === 'custom_jar' ? 'bg-omni-indigo/20 border-omni-indigo text-white font-bold' : 'border-obsidian-750 text-slate-400 hover:border-obsidian-600'}"
              >
                <div>JAR Personalizado</div>
                <div class="text-[9px] text-slate-500 font-normal">Ruta a binario local</div>
              </button>
            </div>

            {#if executableType === 'jar_bundle'}
              <div class="pt-1">
                <span class="text-[11px] text-slate-400 block mb-1">Seleccionar Lanzador Embebido (.jar):</span>
                <select
                  bind:value={launcherJar}
                  class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none"
                >
                  {#each launchers as l}
                    <option value={l}>{l}</option>
                  {/each}
                </select>
              </div>
            {:else if executableType === 'custom_jar'}
              <div class="pt-1">
                <span class="text-[11px] text-slate-400 block mb-1">Ruta absoluta al JAR personalizado:</span>
                <input
                  type="text"
                  bind:value={customExec}
                  placeholder="/ruta/a/mi-cliente.jar"
                  class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none"
                />
              </div>
            {/if}
          </div>
        {/if}

        <!-- Configuración de Java y Memoria RAM (En Modo Básico solo muestra el slider de RAM, en Avanzado muestra flags y presets) -->
        <div class="p-3.5 bg-obsidian-950/60 border border-obsidian-800 rounded-xl space-y-3">
          <label class="block text-slate-300 font-bold flex items-center gap-2">
            <Cpu class="w-4 h-4 text-amber-400" />
            Entorno Java y Asignación de Memoria RAM
          </label>

          {#if uiMode === 'advanced'}
            <div class="grid grid-cols-2 gap-3">
              <div>
                <span class="text-[11px] text-slate-400 block mb-1">Selección de Java:</span>
                <select
                  bind:value={javaMode}
                  class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none"
                >
                  <option value="auto">Automático (Detectar según versión de MC)</option>
                  <option value="custom">Java Específico Manual</option>
                </select>
              </div>

              <div>
                <span class="text-[11px] text-slate-400 block mb-1">Recolector de Basura (GC) & Preset JVM:</span>
                <select
                  bind:value={jvmPreset}
                  class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none"
                >
                  <option value="aikar">Aikar Flags G1GC (Recomendado Alto Rendimiento y Servidores)</option>
                  <option value="generational_zgc">Generational ZGC (Java 21/25 Pausas sub-milisegundo)</option>
                  <option value="zgc">ZGC Clásico (Ultra baja latencia en heaps grandes)</option>
                  <option value="shenandoah">Shenandoah Adaptive GC (Suavidad continua de FPS)</option>
                  <option value="graalvm_throughput">GraalVM Throughput (Máxima tasa de ticks)</option>
                  <option value="low_ram">Serial GC (Bajo consumo de memoria RAM / PCs modestas)</option>
                  <option value="standard">Vanilla Puro (Sin optimizaciones extras)</option>
                </select>
              </div>
            </div>

            <!-- Inyector y Plantillas Rápidas de Argumentos JVM -->
            <div class="space-y-1.5">
              <div class="flex items-center justify-between">
                <span class="text-[11px] text-slate-300 font-bold">Parámetros JVM Personalizados:</span>
                <div class="flex items-center gap-1.5 text-[10px]">
                  <span class="text-slate-500">Insertar flag:</span>
                  <button
                    type="button"
                    onclick={() => jvmArgs = (jvmArgs + ' -XX:+UseLargePages').trim()}
                    class="px-1.5 py-0.5 rounded bg-obsidian-800 hover:bg-obsidian-700 text-slate-300 border border-obsidian-700 font-mono"
                    title="Páginas de memoria grande del kernel de Linux"
                  >
                    LargePages
                  </button>
                  <button
                    type="button"
                    onclick={() => jvmArgs = (jvmArgs + ' -XX:+DisableExplicitGC').trim()}
                    class="px-1.5 py-0.5 rounded bg-obsidian-800 hover:bg-obsidian-700 text-slate-300 border border-obsidian-700 font-mono"
                    title="Evita System.gc() forzados por mods"
                  >
                    NoExplicitGC
                  </button>
                  <button
                    type="button"
                    onclick={() => jvmArgs = (jvmArgs + ' -Dsun.java2d.opengl=true').trim()}
                    class="px-1.5 py-0.5 rounded bg-obsidian-800 hover:bg-obsidian-700 text-slate-300 border border-obsidian-700 font-mono"
                    title="Aceleración OpenGL en GUI 2D"
                  >
                    OpenGL2D
                  </button>
                  {#if targetJavaVersion <= 8}
                    <button
                      type="button"
                      onclick={() => jvmArgs = (jvmArgs + ' -XX:MaxPermSize=256m').trim()}
                      class="px-1.5 py-0.5 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/30 font-mono"
                      title="PermSize para Minecraft 1.7.10 y Java 8"
                    >
                      PermSize256m
                    </button>
                  {:else}
                    <button
                      type="button"
                      onclick={() => jvmArgs = (jvmArgs + ' --add-modules=jdk.incubator.vector').trim()}
                      class="px-1.5 py-0.5 rounded bg-omni-indigo/20 hover:bg-omni-indigo/30 text-omni-lavender border border-omni-indigo/30 font-mono"
                      title="SIMD / Vector API para Java moderno"
                    >
                      VectorAPI
                    </button>
                  {/if}
                  <button
                    type="button"
                    onclick={() => jvmArgs = ''}
                    class="px-1.5 py-0.5 rounded bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/20"
                    title="Limpiar argumentos"
                  >
                    Limpiar
                  </button>
                </div>
              </div>
              <textarea
                bind:value={jvmArgs}
                rows="2"
                placeholder="-XX:+AlwaysPreTouch -Dsun.rmi.dgc.server.gcInterval=2147483646..."
                class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg p-2 text-xs font-mono text-omni-cyan focus:outline-none resize-none leading-relaxed"
              ></textarea>
              <p class="text-[10px] text-slate-500">
                Se concatenan automáticamente con los límites de memoria <strong>-Xms{ramMin}M -Xmx{ramMax}M</strong> y el preset de recolector seleccionado.
              </p>
            </div>

            {#if javaMode === 'custom'}
              <div>
                <span class="text-[11px] text-slate-400 block mb-1">Java Runtime Detectado:</span>
                <select
                  bind:value={customJava}
                  class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none"
                >
                  {#each javas as jv}
                    <option value={jv.path}>Java {jv.version} ({jv.vendor}) - {jv.path}</option>
                  {/each}
                </select>
              </div>
            {/if}
          {:else}
            <div class="bg-obsidian-900/60 p-2.5 rounded-lg border border-obsidian-800 text-[11px] text-slate-400 flex items-center justify-between">
              <span>Ajuste automático según Minecraft {mcVersion}: Entorno <strong>Java {targetJavaVersion}</strong>, recolector optimizado y blindaje Log4j aplicados automáticamente.</span>
              <span class="text-omni-cyan font-bold font-mono px-2 py-0.5 rounded bg-omni-cyan/10 border border-omni-cyan/20">Java {targetJavaVersion} Auto</span>
            </div>
          {/if}

          <!-- RAM Slider / Inputs -->

          <!-- RAM Slider / Inputs -->
          <div class="grid grid-cols-2 gap-3 pt-1">
            <div>
              <div class="flex justify-between text-[11px] text-slate-400 mb-1">
                <span>RAM Mínima:</span>
                <span class="font-bold text-white font-mono">{ramMin} MB</span>
              </div>
              <input
                type="range"
                min="512"
                max="16384"
                step="512"
                bind:value={ramMin}
                class="w-full accent-omni-indigo cursor-pointer"
              />
            </div>

            <div>
              <div class="flex justify-between text-[11px] text-slate-400 mb-1">
                <span>RAM Máxima:</span>
                <span class="font-bold text-emerald-400 font-mono">{(ramMax / 1024).toFixed(1)} GB ({ramMax} MB)</span>
              </div>
              <input
                type="range"
                min="1024"
                max="32768"
                step="512"
                bind:value={ramMax}
                class="w-full accent-emerald-500 cursor-pointer"
              />
            </div>
          </div>
        </div>

        {#if instanceType === 'server'}
          <div class="p-3 bg-obsidian-950/60 border border-obsidian-800 rounded-xl">
            <label class="block text-slate-300 font-bold mb-1">Puerto del Servidor</label>
            <input
              type="number"
              bind:value={serverPort}
              class="w-full bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-1.5 text-white focus:outline-none font-mono"
            />
          </div>
        {/if}
      </div>

      <!-- Footer Buttons -->
      <div class="px-6 py-4 border-t border-obsidian-750 bg-obsidian-900/90 flex items-center justify-end gap-3">
        <button
          type="button"
          onclick={onClose}
          class="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white hover:bg-obsidian-800 transition"
        >
          Cancelar
        </button>
        <button
          type="button"
          onclick={handleSubmit}
          class="px-5 py-2 rounded-xl text-xs font-bold bg-omni-indigo hover:bg-omni-indigo/80 text-white shadow-lg shadow-omni-indigo/25 transition flex items-center gap-2"
        >
          <Check class="w-4 h-4" />
          {mode === 'create' ? 'Crear Instancia' : 'Guardar Cambios'}
        </button>
      </div>
    </div>
  </div>
{/if}
