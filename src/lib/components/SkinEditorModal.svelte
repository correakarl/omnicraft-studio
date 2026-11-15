<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    X,
    Paintbrush,
    Eraser,
    PaintBucket,
    Pipette,
    Undo2,
    Redo2,
    Download,
    Upload,
    RotateCw,
    Layers,
    Save,
    ArrowUpRight,
    Sparkles,
    User,
    Check,
    ZoomIn,
    ZoomOut,
    Grid,
    Eye,
    EyeOff,
    Play,
    Pause,
    Search,
    Globe
  } from 'lucide-svelte';
  import { SkinViewer, WalkingAnimation } from 'skinview3d';
  import { invoke } from '@tauri-apps/api/core';
  import type { LocalSkinModel, AccountModel, RemotePublicSkinModel } from '../types';
  import { OFFICIAL_STEVE_BASE64, OFFICIAL_ALEX_BASE64 } from '../officialSkins';

  let {
    isOpen = false,
    activeAccount = null as AccountModel | null,
    onClose = () => {},
    onSaveSkin = (skin: LocalSkinModel) => {},
    onInjectOffline = (base64: string) => {},
    onUploadMojang = (base64: string, modelType: string) => {},
    localSkins = [] as LocalSkinModel[],
    onDeleteSkin = (id: string) => {},
    onRefreshSkins = () => {},
  } = $props<{
    isOpen: boolean;
    activeAccount?: AccountModel | null;
    onClose: () => void;
    onSaveSkin: (skin: LocalSkinModel) => void;
    onInjectOffline: (base64: string) => void;
    onUploadMojang: (base64: string, modelType: string) => void;
    localSkins?: LocalSkinModel[];
    onDeleteSkin?: (id: string) => void;
    onRefreshSkins?: () => void;
  }>();

  // Vista activa: 'editor' | 'library' | 'remote'
  let activeView = $state<'editor' | 'library' | 'remote'>('editor');

  // Estados de la Biblioteca Pública Remota (MineSkin + Perfiles Oficiales Mojang)
  let remoteSkins = $state<RemotePublicSkinModel[]>([]);
  let remoteSearchQuery = $state('');
  let remotePage = $state(1);
  let isSearchingRemote = $state(false);
  let downloadingRemoteIds = $state<Set<string>>(new Set());
  let savedRemoteIds = $state<Set<string>>(new Set());

  async function searchRemoteSkins(page: number = 1) {
    isSearchingRemote = true;
    remotePage = page;
    try {
      const results = await invoke<RemotePublicSkinModel[]>('search_remote_skins_cmd', {
        query: remoteSearchQuery,
        page: page,
      });
      remoteSkins = results;
    } catch (e) {
      console.error('Error buscando skins remotas:', e);
    } finally {
      isSearchingRemote = false;
    }
  }

  async function handleDownloadToLocalLibrary(rSkin: RemotePublicSkinModel) {
    if (downloadingRemoteIds.has(rSkin.id)) return;
    downloadingRemoteIds.add(rSkin.id);
    downloadingRemoteIds = new Set(downloadingRemoteIds);
    try {
      const b64 = await invoke<string>('download_remote_skin_cmd', {
        textureUrl: rSkin.texture_url,
      });
      const localId = `skin_${rSkin.id.replace(/[^a-zA-Z0-9_-]/g, '')}_${Date.now()}`;
      onSaveSkin({
        id: localId,
        name: rSkin.name,
        model_type: (rSkin.model_type === 'slim' ? 'slim' : 'classic'),
        png_data_base64: b64,
      });
      savedRemoteIds.add(rSkin.id);
      savedRemoteIds = new Set(savedRemoteIds);
      onRefreshSkins();
    } catch (e) {
      alert(`Error descargando skin remota: ${e}`);
    } finally {
      downloadingRemoteIds.delete(rSkin.id);
      downloadingRemoteIds = new Set(downloadingRemoteIds);
    }
  }

  async function handleLoadRemoteInEditor(rSkin: RemotePublicSkinModel) {
    if (downloadingRemoteIds.has(rSkin.id)) return;
    downloadingRemoteIds.add(rSkin.id);
    downloadingRemoteIds = new Set(downloadingRemoteIds);
    try {
      const b64 = await invoke<string>('download_remote_skin_cmd', {
        textureUrl: rSkin.texture_url,
      });
      skinName = rSkin.name;
      modelType = rSkin.model_type === 'slim' ? 'slim' : 'classic';
      activeView = 'editor';
      setTimeout(() => {
        if (!canvasEl) return;
        const ctx = canvasEl.getContext('2d');
        if (!ctx) return;
        const img = new Image();
        img.onload = () => {
          ctx.clearRect(0, 0, 64, 64);
          ctx.drawImage(img, 0, 0, 64, 64);
          pushHistory();
          init3DViewer();
        };
        img.src = b64;
      }, 60);
    } catch (e) {
      alert(`Error cargando skin remota en el editor: ${e}`);
    } finally {
      downloadingRemoteIds.delete(rSkin.id);
      downloadingRemoteIds = new Set(downloadingRemoteIds);
    }
  }

  // Canvas y 3D
  let canvasEl = $state<HTMLCanvasElement | undefined>(undefined);
  let skinViewer3dEl = $state<HTMLCanvasElement | undefined>(undefined);
  let viewer3d: SkinViewer | null = null;

  // Estado del editor
  let skinName = $state('Skin Personalizada');
  let modelType = $state<'classic' | 'slim'>('classic');
  let currentTool = $state<'brush' | 'eraser' | 'bucket' | 'picker'>('brush');
  let currentColor = $state('#06b6d4');
  let isDrawing = $state(false);
  let isWalking = $state(false);
  let isLoadingAccountSkin = $state(false);
  let showGrid = $state(true);
  let overlayCanvasEl = $state<HTMLCanvasElement | undefined>(undefined);

  // Selector de Capa Activa (Inner / Outer Layer)
  let activeLayer = $state<'inner' | 'outer' | 'both'>('both');

  // Visibilidad de partes del cuerpo (como Skindex)
  let visibleParts = $state({
    head: true,
    body: true,
    leftArm: true,
    rightArm: true,
    leftLeg: true,
    rightLeg: true
  });

  // Definición UV canónica de Minecraft 64x64 para cada parte y capa
  type BodyPartKey = 'head' | 'body' | 'leftArm' | 'rightArm' | 'leftLeg' | 'rightLeg';
  interface UVRegion {
    part: BodyPartKey;
    layer: 'inner' | 'outer';
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  }

  const UV_REGIONS: UVRegion[] = [
    // Cabeza
    { part: 'head', layer: 'inner', x1: 0, y1: 0, x2: 32, y2: 16 },
    { part: 'head', layer: 'outer', x1: 32, y1: 0, x2: 64, y2: 16 },

    // Torso / Cuerpo
    { part: 'body', layer: 'inner', x1: 16, y1: 16, x2: 40, y2: 32 },
    { part: 'body', layer: 'outer', x1: 16, y1: 32, x2: 40, y2: 48 },

    // Brazo Derecho
    { part: 'rightArm', layer: 'inner', x1: 40, y1: 16, x2: 56, y2: 32 },
    { part: 'rightArm', layer: 'outer', x1: 40, y1: 32, x2: 56, y2: 48 },

    // Pierna Derecha
    { part: 'rightLeg', layer: 'inner', x1: 0, y1: 16, x2: 16, y2: 32 },
    { part: 'rightLeg', layer: 'outer', x1: 0, y1: 32, x2: 16, y2: 48 },

    // Brazo Izquierdo
    { part: 'leftArm', layer: 'inner', x1: 32, y1: 48, x2: 48, y2: 64 },
    { part: 'leftArm', layer: 'outer', x1: 48, y1: 48, x2: 64, y2: 64 },

    // Pierna Izquierda
    { part: 'leftLeg', layer: 'inner', x1: 16, y1: 48, x2: 32, y2: 64 },
    { part: 'leftLeg', layer: 'outer', x1: 0, y1: 48, x2: 16, y2: 64 },
  ];

  function getPixelZone(x: number, y: number): { part: BodyPartKey; layer: 'inner' | 'outer' } | null {
    for (const reg of UV_REGIONS) {
      if (x >= reg.x1 && x < reg.x2 && y >= reg.y1 && y < reg.y2) {
        return { part: reg.part, layer: reg.layer };
      }
    }
    return null;
  }

  function isPixelEditable(x: number, y: number): boolean {
    const zone = getPixelZone(x, y);
    // Si cae fuera de cualquier parte registrada, permitir edición o no según capa
    if (!zone) return true;

    // 1. Comprobar si la parte del cuerpo está visible
    if (!visibleParts[zone.part]) return false;

    // 2. Comprobar si la capa está activa
    if (activeLayer === 'inner' && zone.layer !== 'inner') return false;
    if (activeLayer === 'outer' && zone.layer !== 'outer') return false;

    return true;
  }

  function drawMaskOverlay() {
    if (!overlayCanvasEl) return;
    const ctx = overlayCanvasEl.getContext('2d');
    if (!ctx) return;
    ctx.clearRect(0, 0, 64, 64);

    // Oscurecer y bloquear visualmente los sectores que NO son editables
    for (let y = 0; y < 64; y++) {
      for (let x = 0; x < 64; x++) {
        if (!isPixelEditable(x, y)) {
          ctx.fillStyle = 'rgba(15, 23, 42, 0.72)';
          ctx.fillRect(x, y, 1, 1);
        }
      }
    }
  }

  // Historial Deshacer / Rehacer
  let historyStack = $state<ImageData[]>([]);
  let historyIndex = $state(-1);

  // Paleta predefinida Skindex
  const skindexPalette = [
    '#000000', '#2d3748', '#4a5568', '#718096', '#cbd5e0', '#ffffff',
    '#744210', '#975a16', '#b7791f', '#d69e2e', '#ecc94b', '#fefcbf',
    '#9b2c2c', '#c53030', '#e53e3e', '#f56565', '#feb2b2', '#fff5f5',
    '#22543d', '#276749', '#2f855a', '#38a169', '#48bb78', '#9ae6b4',
    '#1a365d', '#2a4365', '#2b6cb0', '#3182ce', '#4299e1', '#bee3f8',
    '#44337a', '#553c9a', '#6b46c1', '#805ad5', '#9f7aea', '#e9d8fd',
    '#702459', '#97266d', '#b83280', '#d53f8c', '#ed64a6', '#fbb6ce'
  ];

  function pushHistory() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    const state = ctx.getImageData(0, 0, 64, 64);
    // Eliminar estados futuros si estamos en medio del historial
    if (historyIndex < historyStack.length - 1) {
      historyStack = historyStack.slice(0, historyIndex + 1);
    }
    historyStack.push(state);
    if (historyStack.length > 25) {
      historyStack.shift();
    } else {
      historyIndex++;
    }
  }

  function handleUndo() {
    if (historyIndex > 0 && canvasEl) {
      historyIndex--;
      const ctx = canvasEl.getContext('2d');
      if (ctx) {
        ctx.putImageData(historyStack[historyIndex], 0, 0);
        update3DPreview();
      }
    }
  }

  function handleRedo() {
    if (historyIndex < historyStack.length - 1 && canvasEl) {
      historyIndex++;
      const ctx = canvasEl.getContext('2d');
      if (ctx) {
        ctx.putImageData(historyStack[historyIndex], 0, 0);
        update3DPreview();
      }
    }
  }

  async function initCanvas() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;

    // Precargar la skin asignada al perfil seleccionado (sea Non-Premium o Microsoft)
    if (activeAccount && activeAccount.username) {
      isLoadingAccountSkin = true;
      try {
        const res = await invoke<[string, string] | null>('get_profile_skin_cmd', {
          username: activeAccount.username,
          uuid: activeAccount.uuid || null,
          accountType: activeAccount.account_type || null,
        });

        if (res && res[0]) {
          const [skinB64, detectedModel] = res;
          if (detectedModel === 'slim' || detectedModel === 'classic') {
            modelType = detectedModel;
          }

          const skinImg = new Image();
          skinImg.onload = () => {
            if (!ctx || !canvasEl) return;
            ctx.clearRect(0, 0, 64, 64);
            ctx.drawImage(skinImg, 0, 0, 64, 64);
            historyStack = [];
            historyIndex = -1;
            pushHistory();
            init3DViewer();
            isLoadingAccountSkin = false;
          };
          skinImg.onerror = () => {
            fallbackToOfficialDefault(ctx);
            isLoadingAccountSkin = false;
          };
          skinImg.src = skinB64;
          return;
        }
      } catch (err) {
        console.warn('No se pudo precargar la skin del perfil, usando plantilla por defecto:', err);
      } finally {
        isLoadingAccountSkin = false;
      }
    }

    fallbackToOfficialDefault(ctx);
  }

  function fallbackToOfficialDefault(ctx: CanvasRenderingContext2D) {
    // Cargar la textura oficial de Mojang auténtica (Steve o Alex 64x64 HD)
    const officialImg = new Image();
    officialImg.onload = () => {
      if (!ctx || !canvasEl) return;
      ctx.clearRect(0, 0, 64, 64);
      ctx.drawImage(officialImg, 0, 0, 64, 64);
      historyStack = [];
      historyIndex = -1;
      pushHistory();
      update3DPreview();
    };
    officialImg.src = modelType === 'slim' ? OFFICIAL_ALEX_BASE64 : OFFICIAL_STEVE_BASE64;
  }

  function toggleWalking() {
    isWalking = !isWalking;
    if (viewer3d) {
      if (isWalking) {
        const anim = new WalkingAnimation();
        anim.speed = 0.6;
        viewer3d.animation = anim;
      } else {
        viewer3d.animation = null;
      }
    }
  }

  function init3DViewer() {
    if (!skinViewer3dEl) return;
    if (viewer3d) {
      viewer3d.dispose();
      viewer3d = null;
    }

    viewer3d = new SkinViewer({
      canvas: skinViewer3dEl,
      width: 320,
      height: 380,
      model: modelType === 'slim' ? 'slim' : 'default',
    });

    viewer3d.fov = 60;
    viewer3d.zoom = 0.95;
    viewer3d.controls.enableRotate = true;
    viewer3d.controls.enableZoom = true;

    if (isWalking) {
      const anim = new WalkingAnimation();
      anim.speed = 0.6;
      viewer3d.animation = anim;
    } else {
      viewer3d.animation = null;
    }

    update3DPreview();
    applyPartVisibility();
  }

  function applyPartVisibility() {
    drawMaskOverlay();

    if (!viewer3d || !viewer3d.playerObject) return;
    const skin = viewer3d.playerObject.skin;

    // Aplicar visibilidad de extremidades
    skin.head.visible = visibleParts.head;
    skin.body.visible = visibleParts.body;
    skin.leftArm.visible = visibleParts.leftArm;
    skin.rightArm.visible = visibleParts.rightArm;
    skin.leftLeg.visible = visibleParts.leftLeg;
    skin.rightLeg.visible = visibleParts.rightLeg;

    // Control de Capas: Piel Base vs Capa Exterior (Hat/Jacket)
    const showInner = activeLayer === 'inner' || activeLayer === 'both';
    const showOuter = activeLayer === 'outer' || activeLayer === 'both';

    if (skin.head) {
      if ((skin.head as any).innerLayer) (skin.head as any).innerLayer.visible = showInner;
      if ((skin.head as any).outerLayer) (skin.head as any).outerLayer.visible = showOuter;
    }
    if (skin.body) {
      if ((skin.body as any).innerLayer) (skin.body as any).innerLayer.visible = showInner;
      if ((skin.body as any).outerLayer) (skin.body as any).outerLayer.visible = showOuter;
    }
    if (skin.leftArm) {
      if ((skin.leftArm as any).innerLayer) (skin.leftArm as any).innerLayer.visible = showInner;
      if ((skin.leftArm as any).outerLayer) (skin.leftArm as any).outerLayer.visible = showOuter;
    }
    if (skin.rightArm) {
      if ((skin.rightArm as any).innerLayer) (skin.rightArm as any).innerLayer.visible = showInner;
      if ((skin.rightArm as any).outerLayer) (skin.rightArm as any).outerLayer.visible = showOuter;
    }
    if (skin.leftLeg) {
      if ((skin.leftLeg as any).innerLayer) (skin.leftLeg as any).innerLayer.visible = showInner;
      if ((skin.leftLeg as any).outerLayer) (skin.leftLeg as any).outerLayer.visible = showOuter;
    }
    if (skin.rightLeg) {
      if ((skin.rightLeg as any).innerLayer) (skin.rightLeg as any).innerLayer.visible = showInner;
      if ((skin.rightLeg as any).outerLayer) (skin.rightLeg as any).outerLayer.visible = showOuter;
    }
  }

  function update3DPreview() {
    if (!canvasEl || !viewer3d) return;
    const dataUrl = canvasEl.toDataURL('image/png');
    viewer3d.loadSkin(dataUrl, {
      model: modelType === 'slim' ? 'slim' : 'default',
    });
  }

  function handlePixelAction(x: number, y: number) {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    if (currentTool === 'picker') {
      const pixel = ctx.getImageData(x, y, 1, 1).data;
      if (pixel[3] > 0) {
        currentColor = `#${((1 << 24) + (pixel[0] << 16) + (pixel[1] << 8) + pixel[2]).toString(16).slice(1)}`;
      }
      currentTool = 'brush';
      return;
    }

    // Verificar si el píxel pertenece a una parte o capa actualmente visible y editable
    if (!isPixelEditable(x, y)) {
      return;
    }

    if (currentTool === 'brush') {
      ctx.fillStyle = currentColor;
      ctx.fillRect(x, y, 1, 1);
    } else if (currentTool === 'eraser') {
      ctx.clearRect(x, y, 1, 1);
    } else if (currentTool === 'bucket') {
      floodFill(ctx, x, y, currentColor);
    }

    update3DPreview();
  }

  function floodFill(ctx: CanvasRenderingContext2D, startX: number, startY: number, fillHex: string) {
    if (!isPixelEditable(startX, startY)) return;

    const imgData = ctx.getImageData(0, 0, 64, 64);
    const data = imgData.data;
    const startIndex = (startY * 64 + startX) * 4;
    const targetR = data[startIndex];
    const targetG = data[startIndex + 1];
    const targetB = data[startIndex + 2];
    const targetA = data[startIndex + 3];

    // Convertir hex a RGB
    const fillR = parseInt(fillHex.slice(1, 3), 16);
    const fillG = parseInt(fillHex.slice(3, 5), 16);
    const fillB = parseInt(fillHex.slice(5, 7), 16);
    const fillA = 255;

    if (targetR === fillR && targetG === fillG && targetB === fillB && targetA === fillA) return;

    const queue: [number, number][] = [[startX, startY]];
    const seen = new Uint8Array(64 * 64);

    while (queue.length > 0) {
      const [cx, cy] = queue.pop()!;
      if (cx < 0 || cx >= 64 || cy < 0 || cy >= 64) continue;
      const idx = cy * 64 + cx;
      if (seen[idx]) continue;
      seen[idx] = 1;

      // Respetar las zonas visibles en el relleno por cubeta
      if (!isPixelEditable(cx, cy)) continue;

      const pIdx = idx * 4;
      if (
        data[pIdx] === targetR &&
        data[pIdx + 1] === targetG &&
        data[pIdx + 2] === targetB &&
        data[pIdx + 3] === targetA
      ) {
        data[pIdx] = fillR;
        data[pIdx + 1] = fillG;
        data[pIdx + 2] = fillB;
        data[pIdx + 3] = fillA;

        queue.push([cx + 1, cy]);
        queue.push([cx - 1, cy]);
        queue.push([cx, cy + 1]);
        queue.push([cx, cy - 1]);
      }
    }
    ctx.putImageData(imgData, 0, 0);
  }

  function handleCanvasMouseEvent(e: MouseEvent) {
    if (!canvasEl) return;
    if (e.type === 'mousedown') {
      isDrawing = true;
    } else if (e.type === 'mouseup' || e.type === 'mouseleave') {
      if (isDrawing) {
        isDrawing = false;
        pushHistory();
      }
      return;
    }

    if (!isDrawing && e.type !== 'click') return;

    const rect = canvasEl.getBoundingClientRect();
    const x = Math.floor(((e.clientX - rect.left) / rect.width) * 64);
    const y = Math.floor(((e.clientY - rect.top) / rect.height) * 64);

    if (x >= 0 && x < 64 && y >= 0 && y < 64) {
      handlePixelAction(x, y);
    }
  }

  function handleImportPng(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file || !canvasEl) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const img = new Image();
      img.onload = () => {
        const ctx = canvasEl!.getContext('2d');
        if (!ctx) return;
        ctx.clearRect(0, 0, 64, 64);
        ctx.drawImage(img, 0, 0, 64, 64);
        pushHistory();
        update3DPreview();
      };
      img.src = ev.target?.result as string;
    };
    reader.readAsDataURL(file);
  }

  function handleExportPng() {
    if (!canvasEl) return;
    const link = document.createElement('a');
    link.download = `${skinName.toLowerCase().replace(/\s+/g, '_')}.png`;
    link.href = canvasEl.toDataURL('image/png');
    link.click();
  }

  // Observar cambios de cuenta/perfil seleccionado mientras el modal esté abierto para actualizar el Editor 3D/2D
  let lastLoadedProfileUsername = $state<string>('');
  $effect(() => {
    const currentUsername = activeAccount?.username || '';
    if (isOpen && currentUsername && currentUsername !== lastLoadedProfileUsername) {
      lastLoadedProfileUsername = currentUsername;
      setTimeout(() => {
        initCanvas();
        init3DViewer();
        drawMaskOverlay();
      }, 60);
    } else if (!isOpen) {
      lastLoadedProfileUsername = '';
      if (viewer3d) {
        viewer3d.dispose();
        viewer3d = null;
      }
    }
  });

  onDestroy(() => {
    if (viewer3d) viewer3d.dispose();
  });
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 bg-black/85 backdrop-blur-md animate-in fade-in duration-150">
    <div class="bg-obsidian-900 border border-obsidian-750/90 rounded-2xl w-full max-w-6xl h-[92vh] flex flex-col shadow-2xl overflow-hidden">
      <!-- Barra Superior Estilo Skindex Studio -->
      <div class="px-5 py-3 border-b border-obsidian-750 bg-obsidian-950/90 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-omni-indigo to-omni-cyan flex items-center justify-center font-bold text-white shadow">
            🎨
          </div>
          <div>
            <h2 class="text-sm font-bold text-white flex items-center gap-2">
              Omnicraft Skin Studio 3D
              <span class="text-[10px] px-1.5 py-0.2 rounded bg-omni-cyan/15 text-omni-cyan font-mono border border-omni-cyan/30">Skindex Pro Engine</span>
            </h2>
            <p class="text-[11px] text-slate-400">Editor 3D interactivo con mapa UV, capas y aislamiento de extremidades</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <!-- Pestañas de Vista: Editor vs Biblioteca Local vs Catálogo Público Remoto -->
          <div class="flex bg-obsidian-900 border border-obsidian-750 p-0.5 rounded-lg mr-2">
            <button
              type="button"
              onclick={() => { activeView = 'editor'; setTimeout(init3DViewer, 50); }}
              class="px-2.5 py-1 text-xs font-bold rounded-md transition {activeView === 'editor' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
            >
              Editor 3D
            </button>
            <button
              type="button"
              onclick={() => { activeView = 'library'; onRefreshSkins(); }}
              class="px-2.5 py-1 text-xs font-bold rounded-md transition flex items-center gap-1.5 {activeView === 'library' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
            >
              <span>Biblioteca Local</span>
              <span class="text-[10px] px-1 py-0.2 rounded-full bg-black/40 font-mono text-omni-cyan">{localSkins.length}</span>
            </button>
            <button
              type="button"
              onclick={() => { activeView = 'remote'; if (remoteSkins.length === 0) searchRemoteSkins(1); }}
              class="px-2.5 py-1 text-xs font-bold rounded-md transition flex items-center gap-1.5 {activeView === 'remote' ? 'bg-omni-indigo text-white shadow' : 'text-slate-400 hover:text-white'}"
            >
              <Globe class="w-3.5 h-3.5 text-emerald-400" />
              <span>Explorar Catálogo Público</span>
            </button>
          </div>

          <!-- Notificación / Badge de cuenta activa vinculada -->
          {#if activeAccount}
            <div class="px-2.5 py-1 rounded-lg border text-xs flex items-center gap-1.5 {activeAccount.account_type === 'microsoft' ? 'bg-omni-indigo/20 border-omni-indigo/40 text-omni-lavender' : 'bg-obsidian-900 border-obsidian-750 text-slate-300'}">
              <User class="w-3.5 h-3.5 {activeAccount.account_type === 'microsoft' ? 'text-omni-cyan' : 'text-slate-400'}" />
              <span class="font-bold">{activeAccount.username}</span>
              {#if activeAccount.account_type === 'microsoft'}
                <span class="text-[9px] bg-omni-cyan/20 text-omni-cyan px-1 rounded font-mono">Premium</span>
              {/if}
            </div>
          {/if}

          <!-- Entrada rápida del nombre -->
          <input
            type="text"
            bind:value={skinName}
            placeholder="Nombre de la skin"
            class="bg-obsidian-900 border border-obsidian-700 rounded-lg px-2.5 py-1 text-xs text-white focus:outline-none focus:border-omni-cyan w-44 font-medium"
          />

          <!-- Botón Cerrar -->
          <button onclick={onClose} class="text-slate-400 hover:text-white p-1.5 rounded-lg hover:bg-obsidian-800 transition">
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- Área de Trabajo Principal (3 Columnas Ergonómicas o Vista Biblioteca) -->
      {#if activeView === 'library'}
        <!-- VISTA BIBLIOTECA DE SKINS -->
        <div class="flex-1 bg-obsidian-950 p-6 overflow-y-auto">
          <div class="max-w-5xl mx-auto space-y-6">
            <div class="flex items-center justify-between pb-3 border-b border-obsidian-800">
              <div>
                <h3 class="text-base font-bold text-white flex items-center gap-2">
                  <Sparkles class="w-4 h-4 text-omni-cyan" />
                  Biblioteca de Skins Registradas ({localSkins.length})
                </h3>
                <p class="text-xs text-slate-400 mt-0.5">
                  Previsualiza y aplica tus apariencias creadas o importadas directamente en tus instancias.
                </p>
              </div>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={() => { activeView = 'remote'; if (remoteSkins.length === 0) searchRemoteSkins(1); }}
                  class="px-3.5 py-1.5 bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-500/40 text-emerald-300 font-bold rounded-lg text-xs transition flex items-center gap-1.5 shadow"
                >
                  <Globe class="w-3.5 h-3.5 text-emerald-400" /> Explorar Catálogo Público Online
                </button>
                <button
                  type="button"
                  onclick={() => { activeView = 'editor'; setTimeout(init3DViewer, 50); }}
                  class="px-3.5 py-1.5 bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold rounded-lg text-xs transition flex items-center gap-1.5 shadow"
                >
                  <Paintbrush class="w-3.5 h-3.5" /> Crear / Editar Nueva Skin
                </button>
              </div>
            </div>

            {#if localSkins.length === 0}
              <div class="text-center py-16 bg-obsidian-900/50 border border-obsidian-800 rounded-2xl p-8 space-y-3">
                <div class="w-12 h-12 rounded-2xl bg-obsidian-800 flex items-center justify-center mx-auto text-slate-400">
                  🎨
                </div>
                <h4 class="text-sm font-bold text-white">No hay skins en la biblioteca</h4>
                <p class="text-xs text-slate-400 max-w-md mx-auto">
                  Diseña una apariencia en el editor 3D o importa un archivo PNG para almacenarlo en la base de datos local SQLite.
                </p>
                <button
                  type="button"
                  onclick={() => { activeView = 'editor'; setTimeout(init3DViewer, 50); }}
                  class="px-4 py-2 bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold rounded-xl text-xs transition inline-flex items-center gap-2"
                >
                  Abrir Editor 3D
                </button>
              </div>
            {:else}
              <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                {#each localSkins as s (s.id)}
                  <div class="p-3 bg-obsidian-900/80 border border-obsidian-750/80 hover:border-omni-indigo/60 rounded-xl flex flex-col justify-between space-y-3 transition shadow-sm group">
                    <div class="space-y-2">
                      <!-- Renderizado 2D Texture Preview -->
                      <div class="h-32 bg-obsidian-950 rounded-lg border border-obsidian-800 flex items-center justify-center p-2 overflow-hidden relative" style="background: repeating-conic-gradient(#111420 0% 25%, #181c2e 0% 50%) 50% / 10px 10px;">
                        <img
                          src={s.png_data_base64}
                          alt={s.name}
                          class="max-h-full max-w-full object-contain filter drop-shadow group-hover:scale-105 transition"
                          style="image-rendering: pixelated;"
                        />
                        <span class="absolute top-1.5 right-1.5 px-1.5 py-0.5 rounded text-[9px] font-mono font-bold bg-obsidian-900/90 border border-obsidian-750 text-slate-300">
                          {s.model_type === 'slim' ? 'Alex (3px)' : 'Steve (4px)'}
                        </span>
                      </div>

                      <div class="min-w-0">
                        <h4 class="text-xs font-bold text-white truncate">{s.name}</h4>
                        <span class="text-[10px] text-slate-400 font-mono">ID: {s.id}</span>
                      </div>
                    </div>

                    <!-- Botones de Acción -->
                    <div class="space-y-1.5 pt-2 border-t border-obsidian-800">
                      <!-- Cargar en el Editor 3D -->
                      <button
                        type="button"
                        onclick={() => {
                          skinName = s.name;
                          modelType = s.model_type as any;
                          activeView = 'editor';
                          setTimeout(() => {
                            if (!canvasEl) return;
                            const ctx = canvasEl.getContext('2d');
                            if (!ctx) return;
                            const img = new Image();
                            img.onload = () => {
                              ctx.clearRect(0, 0, 64, 64);
                              ctx.drawImage(img, 0, 0);
                              init3DViewer();
                            };
                            img.src = s.png_data_base64;
                          }, 60);
                        }}
                        class="w-full py-1.5 px-2 bg-omni-indigo/20 hover:bg-omni-indigo/30 border border-omni-indigo/40 text-omni-lavender font-bold rounded-lg text-[11px] transition flex items-center justify-center gap-1.5"
                      >
                        <Paintbrush class="w-3 h-3 text-omni-cyan" /> Cargar en Editor
                      </button>

                       <div class="grid grid-cols-2 gap-1.5">
                        <!-- Asignar al Perfil Seleccionado -->
                        <button
                          type="button"
                          onclick={() => onInjectOffline(s.png_data_base64)}
                          class="py-1 px-1.5 bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-500/30 text-emerald-300 font-semibold rounded-lg text-[10px] transition text-center truncate"
                          title="Asignar esta skin al perfil seleccionado (se inyectará al juego al arrancar)"
                        >
                          Asignar al Perfil
                        </button>

                        <!-- Subir a Mojang si es Premium -->
                        {#if activeAccount?.account_type === 'microsoft'}
                          <button
                            type="button"
                            onclick={() => onUploadMojang(s.png_data_base64, s.model_type)}
                            class="py-1 px-1.5 bg-omni-cyan/20 hover:bg-omni-cyan/30 border border-omni-cyan/40 text-omni-cyan font-semibold rounded-lg text-[10px] transition text-center truncate"
                            title="Subir a los servidores oficiales de Mojang"
                          >
                            Mojang
                          </button>
                        {:else}
                          <button
                            type="button"
                            onclick={() => onDeleteSkin(s.id)}
                            class="py-1 px-1.5 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 text-rose-400 font-semibold rounded-lg text-[10px] transition text-center"
                            title="Eliminar de la biblioteca local"
                          >
                            Eliminar
                          </button>
                        {/if}
                      </div>

                      {#if activeAccount?.account_type === 'microsoft'}
                        <button
                          type="button"
                          onclick={() => onDeleteSkin(s.id)}
                          class="w-full py-1 text-slate-500 hover:text-rose-400 text-[10px] transition text-center flex items-center justify-center gap-1"
                        >
                          Eliminar skin
                        </button>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {:else if activeView === 'remote'}
        <!-- VISTA CATÁLOGO REMOTO PÚBLICO DE SKINS (MINESKIN + MOJANG) -->
        <div class="flex-1 bg-obsidian-950 p-6 overflow-y-auto">
          <div class="max-w-5xl mx-auto space-y-5">
            <!-- Header y Buscador Remoto -->
            <div class="p-4 rounded-2xl bg-obsidian-900/90 border border-obsidian-750/90 space-y-3 shadow-md">
              <div class="flex flex-col md:flex-row md:items-center justify-between gap-3">
                <div>
                  <h3 class="text-base font-bold text-white flex items-center gap-2">
                    <Globe class="w-4 h-4 text-emerald-400" />
                    Catálogo Público Remoto de Skins (MineSkin & Mojang)
                  </h3>
                  <p class="text-xs text-slate-400 mt-0.5">
                    Busca por temática, personaje o nombre de jugador oficial y descárgalas en 1 clic a tu Biblioteca Local.
                  </p>
                </div>

                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    onclick={() => { activeView = 'library'; onRefreshSkins(); }}
                    class="px-3 py-1.5 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 text-slate-200 border border-obsidian-700 text-xs font-bold transition flex items-center gap-1.5"
                  >
                    <span>Ir a Mi Biblioteca ({localSkins.length})</span>
                  </button>
                </div>
              </div>

              <!-- Barra de Búsqueda + Tags Rápidos -->
              <div class="flex flex-col sm:flex-row gap-2">
                <div class="relative flex-1">
                  <Search class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
                  <input
                    type="text"
                    bind:value={remoteSearchQuery}
                    onkeydown={(e) => { if (e.key === 'Enter') searchRemoteSkins(1); }}
                    placeholder="Buscar skins por tema (ej: knight, anime, cyberpunk, demon) o jugador Mojang (ej: ThaukS, Technoblade)..."
                    class="w-full bg-obsidian-950 border border-obsidian-700 rounded-xl pl-9 pr-3 py-2 text-xs text-white focus:outline-none focus:border-omni-cyan"
                  />
                </div>
                <button
                  type="button"
                  onclick={() => searchRemoteSkins(1)}
                  disabled={isSearchingRemote}
                  class="px-4 py-2 rounded-xl bg-gradient-to-r from-omni-indigo to-omni-cyan hover:brightness-110 text-white font-bold text-xs transition flex items-center justify-center gap-1.5 shadow disabled:opacity-50"
                >
                  {#if isSearchingRemote}
                    <RotateCw class="w-3.5 h-3.5 animate-spin" /> Buscando...
                  {:else}
                    <Search class="w-3.5 h-3.5" /> Consultar Catálogo
                  {/if}
                </button>
              </div>

              <!-- Etiquetas Rápidas de Categorías Populares -->
              <div class="flex flex-wrap items-center gap-1.5 pt-1 text-[11px]">
                <span class="text-slate-400 font-semibold mr-1">Tendencias:</span>
                {#each [
                  { label: '🔥 Recientes', q: '' },
                  { label: '⚔️ Caballero (Knight)', q: 'knight' },
                  { label: '👑 Rey (King)', q: 'king' },
                  { label: '🥷 Ninja / Samurai', q: 'samurai' },
                  { label: '🤖 Cyberpunk', q: 'cyber' },
                  { label: '🐉 Dragón', q: 'dragon' },
                  { label: '🧙 Mago (Mage)', q: 'mage' },
                  { label: '🐷 Technoblade', q: 'Technoblade' },
                ] as tag}
                  <button
                    type="button"
                    onclick={() => { remoteSearchQuery = tag.q; searchRemoteSkins(1); }}
                    class="px-2.5 py-1 rounded-lg border text-[11px] font-medium transition {remoteSearchQuery === tag.q ? 'bg-omni-cyan/20 border-omni-cyan text-omni-cyan font-bold' : 'bg-obsidian-950 border-obsidian-800 text-slate-300 hover:border-obsidian-600'}"
                  >
                    {tag.label}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Resultados del Catálogo Remoto -->
            {#if isSearchingRemote}
              <div class="py-20 text-center space-y-3">
                <RotateCw class="w-8 h-8 text-omni-cyan animate-spin mx-auto" />
                <p class="text-xs font-bold text-slate-300">Consultando bibliotecas públicas remotas de skins...</p>
              </div>
            {:else if remoteSkins.length === 0}
              <div class="text-center py-14 bg-obsidian-900/50 border border-obsidian-800 rounded-2xl p-8 space-y-2">
                <p class="text-sm font-bold text-white">No se encontraron resultados para "{remoteSearchQuery}"</p>
                <p class="text-xs text-slate-400">Prueba con otra palabra clave en inglés (ej. warrior, hoodie, astronaut) o el nombre exacto de un jugador de Minecraft.</p>
              </div>
            {:else}
              <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                {#each remoteSkins as rSkin (rSkin.id)}
                  {@const isDownloading = downloadingRemoteIds.has(rSkin.id)}
                  {@const isSaved = savedRemoteIds.has(rSkin.id)}
                  <div class="p-3 bg-obsidian-900/80 border border-obsidian-750/80 hover:border-omni-cyan/60 rounded-xl flex flex-col justify-between space-y-3 transition shadow-sm group">
                    <div class="space-y-2">
                      <div class="h-36 bg-obsidian-950 rounded-lg border border-obsidian-800 flex items-center justify-center p-2 overflow-hidden relative" style="background: repeating-conic-gradient(#111420 0% 25%, #181c2e 0% 50%) 50% / 10px 10px;">
                        <img
                          src={rSkin.preview_url}
                          alt={rSkin.name}
                          class="max-h-full max-w-full object-contain filter drop-shadow group-hover:scale-105 transition"
                          onerror={(e) => { (e.currentTarget as HTMLImageElement).src = rSkin.texture_url; }}
                        />
                        <span class="absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded text-[9px] font-mono font-bold bg-obsidian-900/90 border border-obsidian-750 text-emerald-400">
                          {rSkin.source}
                        </span>
                      </div>

                      <div class="min-w-0">
                        <h4 class="text-xs font-bold text-white truncate" title={rSkin.name}>{rSkin.name}</h4>
                        <span class="text-[10px] text-slate-400 font-mono truncate block">Textura Oficial 64x64</span>
                      </div>
                    </div>

                    <!-- Acciones: Guardar en Biblioteca Local o Abrir en Editor 3D -->
                    <div class="space-y-1.5 pt-2 border-t border-obsidian-800">
                      <button
                        type="button"
                        disabled={isDownloading}
                        onclick={() => handleDownloadToLocalLibrary(rSkin)}
                        class="w-full py-1.5 px-2 rounded-lg text-[11px] font-bold transition flex items-center justify-center gap-1.5 shadow-sm {isSaved ? 'bg-emerald-500/20 border border-emerald-500/40 text-emerald-300' : 'bg-emerald-600 hover:bg-emerald-500 text-white'}"
                      >
                        {#if isDownloading}
                          <RotateCw class="w-3.5 h-3.5 animate-spin" />
                          <span>Descargando...</span>
                        {:else if isSaved}
                          <Check class="w-3.5 h-3.5" />
                          <span>Guardada en Biblioteca</span>
                        {:else}
                          <Download class="w-3.5 h-3.5" />
                          <span>Guardar en Biblioteca</span>
                        {/if}
                      </button>

                      <button
                        type="button"
                        disabled={isDownloading}
                        onclick={() => handleLoadRemoteInEditor(rSkin)}
                        class="w-full py-1 px-2 rounded-lg bg-obsidian-800 hover:bg-obsidian-750 border border-obsidian-700 text-slate-300 hover:text-white text-[10px] font-semibold transition flex items-center justify-center gap-1"
                      >
                        <Paintbrush class="w-3 h-3 text-omni-cyan" />
                        <span>Abrir en Editor 3D</span>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>

              <!-- Paginación -->
              <div class="flex items-center justify-between pt-4 border-t border-obsidian-800">
                <button
                  type="button"
                  disabled={remotePage <= 1 || isSearchingRemote}
                  onclick={() => searchRemoteSkins(remotePage - 1)}
                  class="px-3.5 py-1.5 rounded-lg bg-obsidian-900 hover:bg-obsidian-800 border border-obsidian-750 text-xs font-bold text-slate-300 disabled:opacity-40 transition"
                >
                  ← Página Anterior
                </button>
                <span class="text-xs font-mono text-slate-400">Página <strong class="text-white">{remotePage}</strong></span>
                <button
                  type="button"
                  disabled={isSearchingRemote}
                  onclick={() => searchRemoteSkins(remotePage + 1)}
                  class="px-3.5 py-1.5 rounded-lg bg-obsidian-900 hover:bg-obsidian-800 border border-obsidian-750 text-xs font-bold text-slate-300 disabled:opacity-40 transition"
                >
                  Siguiente Página →
                </button>
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="flex-1 flex overflow-hidden">
          <!-- Columna Izquierda: Herramientas y Paleta Skindex -->
        <div class="w-64 bg-obsidian-950/70 border-r border-obsidian-750/70 p-3 flex flex-col gap-3 overflow-y-auto shrink-0">
          <!-- Barra de Herramientas de Dibujo -->
          <div class="space-y-1.5">
            <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400">Herramientas</span>
            <div class="grid grid-cols-4 gap-1.5">
              <button
                type="button"
                onclick={() => currentTool = 'brush'}
                class="p-2 rounded-lg border flex flex-col items-center justify-center gap-1 transition {currentTool === 'brush' ? 'bg-omni-cyan text-obsidian-950 font-bold border-omni-cyan shadow' : 'bg-obsidian-900 border-obsidian-750 text-slate-300 hover:text-white'}"
                title="Pincel Píxel a Píxel"
              >
                <Paintbrush class="w-4 h-4" />
                <span class="text-[9px]">Lápiz</span>
              </button>

              <button
                type="button"
                onclick={() => currentTool = 'eraser'}
                class="p-2 rounded-lg border flex flex-col items-center justify-center gap-1 transition {currentTool === 'eraser' ? 'bg-omni-cyan text-obsidian-950 font-bold border-omni-cyan shadow' : 'bg-obsidian-900 border-obsidian-750 text-slate-300 hover:text-white'}"
                title="Borrador transparente"
              >
                <Eraser class="w-4 h-4" />
                <span class="text-[9px]">Borrar</span>
              </button>

              <button
                type="button"
                onclick={() => currentTool = 'bucket'}
                class="p-2 rounded-lg border flex flex-col items-center justify-center gap-1 transition {currentTool === 'bucket' ? 'bg-omni-cyan text-obsidian-950 font-bold border-omni-cyan shadow' : 'bg-obsidian-900 border-obsidian-750 text-slate-300 hover:text-white'}"
                title="Bote de Pintura (Relleno de área)"
              >
                <PaintBucket class="w-4 h-4" />
                <span class="text-[9px]">Bote</span>
              </button>

              <button
                type="button"
                onclick={() => currentTool = 'picker'}
                class="p-2 rounded-lg border flex flex-col items-center justify-center gap-1 transition {currentTool === 'picker' ? 'bg-omni-cyan text-obsidian-950 font-bold border-omni-cyan shadow' : 'bg-obsidian-900 border-obsidian-750 text-slate-300 hover:text-white'}"
                title="Cuentagotas (Tomar color)"
              >
                <Pipette class="w-4 h-4" />
                <span class="text-[9px]">Gotero</span>
              </button>
            </div>

            <!-- Deshacer / Rehacer -->
            <div class="flex gap-1.5 pt-1">
              <button
                type="button"
                onclick={handleUndo}
                disabled={historyIndex <= 0}
                class="flex-1 py-1 rounded bg-obsidian-900 hover:bg-obsidian-850 text-slate-300 text-xs font-semibold flex items-center justify-center gap-1 border border-obsidian-750 disabled:opacity-40"
              >
                <Undo2 class="w-3.5 h-3.5" /> Deshacer
              </button>
              <button
                type="button"
                onclick={handleRedo}
                disabled={historyIndex >= historyStack.length - 1}
                class="flex-1 py-1 rounded bg-obsidian-900 hover:bg-obsidian-850 text-slate-300 text-xs font-semibold flex items-center justify-center gap-1 border border-obsidian-750 disabled:opacity-40"
              >
                <Redo2 class="w-3.5 h-3.5" /> Rehacer
              </button>
            </div>
          </div>

          <!-- Selector de Modelo Steve / Alex -->
          <div class="space-y-1.5">
            <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400">Modelo de Brazos</span>
            <div class="grid grid-cols-2 gap-1.5">
              <button
                type="button"
                onclick={() => { modelType = 'classic'; init3DViewer(); }}
                class="py-1 rounded-lg border text-center text-xs transition {modelType === 'classic' ? 'bg-omni-indigo text-white font-bold border-omni-indigo' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
              >
                Steve (4px)
              </button>
              <button
                type="button"
                onclick={() => { modelType = 'slim'; init3DViewer(); }}
                class="py-1 rounded-lg border text-center text-xs transition {modelType === 'slim' ? 'bg-omni-indigo text-white font-bold border-omni-indigo' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
              >
                Alex (3px)
              </button>
            </div>
          </div>

          <!-- Paleta de Color Skindex Pro -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400">Color Actual</span>
              <div class="flex items-center gap-2">
                <span class="font-mono text-[10px] text-slate-300">{currentColor}</span>
                <input
                  type="color"
                  bind:value={currentColor}
                  class="w-6 h-6 rounded border-none cursor-pointer bg-transparent"
                />
              </div>
            </div>

            <!-- Matriz de Colores Frecuentes -->
            <div class="grid grid-cols-6 gap-1 bg-obsidian-900 p-2 rounded-xl border border-obsidian-750">
              {#each skindexPalette as col}
                <button
                  type="button"
                  title={col}
                  onclick={() => currentColor = col}
                  class="w-5 h-5 rounded border transition hover:scale-110 {currentColor === col ? 'border-white scale-110 ring-1 ring-white' : 'border-obsidian-700'}"
                  style="background-color: {col};"
                ></button>
              {/each}
            </div>
          </div>

          <!-- Selector de Capas (Piel Base / Capa Exterior / Ambas) -->
          <div class="space-y-1.5 pt-1 border-t border-obsidian-800">
            <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400">Capas del Personaje</span>
            <div class="grid grid-cols-3 gap-1 text-[10px]">
              <button
                type="button"
                onclick={() => { activeLayer = 'inner'; applyPartVisibility(); }}
                class="py-1 rounded border text-center font-semibold transition {activeLayer === 'inner' ? 'bg-omni-indigo text-white border-omni-indigo' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
              >
                Piel Base
              </button>
              <button
                type="button"
                onclick={() => { activeLayer = 'outer'; applyPartVisibility(); }}
                class="py-1 rounded border text-center font-semibold transition {activeLayer === 'outer' ? 'bg-omni-indigo text-white border-omni-indigo' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
              >
                2da Capa
              </button>
              <button
                type="button"
                onclick={() => { activeLayer = 'both'; applyPartVisibility(); }}
                class="py-1 rounded border text-center font-semibold transition {activeLayer === 'both' ? 'bg-omni-indigo text-white border-omni-indigo' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
              >
                Ambas
              </button>
            </div>
          </div>

          <!-- Aislamiento de Extremidades (Cuerpo Skindex) -->
          <div class="space-y-1.5 pt-1 border-t border-obsidian-800">
            <div class="flex items-center justify-between">
              <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400">Aislar Partes del Cuerpo</span>
              <button
                type="button"
                onclick={() => {
                  const allVisible = Object.values(visibleParts).every(Boolean);
                  visibleParts.head = !allVisible;
                  visibleParts.body = !allVisible;
                  visibleParts.leftArm = !allVisible;
                  visibleParts.rightArm = !allVisible;
                  visibleParts.leftLeg = !allVisible;
                  visibleParts.rightLeg = !allVisible;
                  applyPartVisibility();
                }}
                class="text-[9px] text-omni-cyan hover:underline"
              >
                {Object.values(visibleParts).every(Boolean) ? 'Ocultar todo' : 'Mostrar todo'}
              </button>
            </div>
            <div class="grid grid-cols-2 gap-1 text-[11px]">
              {#each [
                { id: 'head', label: 'Cabeza' },
                { id: 'body', label: 'Torso' },
                { id: 'leftArm', label: 'Brazo Izq.' },
                { id: 'rightArm', label: 'Brazo Der.' },
                { id: 'leftLeg', label: 'Pierna Izq.' },
                { id: 'rightLeg', label: 'Pierna Der.' },
              ] as part}
                <button
                  type="button"
                  onclick={() => {
                    (visibleParts as any)[part.id] = !(visibleParts as any)[part.id];
                    applyPartVisibility();
                  }}
                  class="py-1 px-2 rounded border text-left flex items-center justify-between transition {(visibleParts as any)[part.id] ? 'bg-obsidian-900 border-obsidian-750 text-white font-medium shadow-sm' : 'bg-obsidian-950/70 border-obsidian-850 text-slate-500 line-through'}"
                  title="{(visibleParts as any)[part.id] ? 'Ocultar y bloquear edición' : 'Mostrar y permitir edición'}"
                >
                  <span class="truncate">{part.label}</span>
                  {#if (visibleParts as any)[part.id]}
                    <Eye class="w-3 h-3 text-omni-cyan shrink-0 ml-1" />
                  {:else}
                    <EyeOff class="w-3 h-3 text-slate-600 shrink-0 ml-1" />
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        </div>

        <!-- Columna Central: Viewport 3D Principal en Escena -->
        <div class="flex-1 bg-obsidian-950 flex flex-col items-center justify-between p-4 relative">
          <!-- Controles Flotantes 3D Superiores -->
          <div class="w-full flex items-center justify-between z-10">
            <div class="flex items-center gap-1.5 bg-obsidian-900/90 border border-obsidian-750 px-2.5 py-1 rounded-lg backdrop-blur">
              <span class="text-xs text-omni-cyan font-bold flex items-center gap-1">
                <Sparkles class="w-3.5 h-3.5" /> Modelo 3D Interactivo
              </span>
              {#if isLoadingAccountSkin}
                <span class="text-[10px] text-amber-300 animate-pulse ml-2">Cargando skin Mojang...</span>
              {/if}
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                onclick={toggleWalking}
                class="px-2.5 py-1 rounded-lg text-xs font-semibold border flex items-center gap-1.5 transition {isWalking ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40 shadow' : 'bg-obsidian-900 border-obsidian-750 text-slate-300 hover:text-white'}"
              >
                {#if isWalking}
                  <Pause class="w-3.5 h-3.5" />
                  <span>Pausar Animación</span>
                {:else}
                  <Play class="w-3.5 h-3.5" />
                  <span>Animar Personaje</span>
                {/if}
              </button>
            </div>
          </div>

          <!-- Canvas 3D Three.js -->
          <div class="flex-1 flex items-center justify-center w-full relative">
            <canvas
              bind:this={skinViewer3dEl}
              class="cursor-grab active:cursor-grabbing rounded-xl drop-shadow-[0_10px_25px_rgba(0,0,0,0.6)]"
            ></canvas>
          </div>

          <!-- Tip inferior -->
          <p class="text-[11px] text-slate-400 font-mono z-10 bg-obsidian-900/80 px-3 py-1 rounded-full border border-obsidian-750">
            Arrastra con el ratón para rotar en 360° • Usa la rueda para hacer zoom
          </p>
        </div>

        <!-- Columna Derecha: Mapa UV 2D de Textura + Exportación / Guardado -->
        <div class="w-80 bg-obsidian-950/70 border-l border-obsidian-750/70 p-4 flex flex-col justify-between shrink-0 overflow-y-auto space-y-4">
          <!-- Lienzo 2D 64x64 Desplegado -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-slate-200">Mapa UV Desplegado (64x64)</span>
              <div class="flex items-center gap-1.5">
                <!-- Botón Toggle Grilla de Píxeles -->
                <button
                  type="button"
                  onclick={() => showGrid = !showGrid}
                  class="px-1.5 py-0.5 rounded text-[10px] font-semibold flex items-center gap-1 border transition {showGrid ? 'bg-omni-cyan/20 border-omni-cyan text-omni-cyan' : 'bg-obsidian-900 border-obsidian-750 text-slate-400 hover:text-white'}"
                  title="Alternar vista de cuadrícula/grilla de píxeles"
                >
                  <Grid class="w-3 h-3" />
                  <span>Grilla</span>
                </button>

                <button
                  type="button"
                  onclick={initCanvas}
                  class="text-[10px] text-amber-400 hover:text-amber-300 font-medium ml-1"
                >
                  Reiniciar
                </button>
              </div>
            </div>

            <!-- Canvas 2D con Zoom Pixel-Art y Grilla Superpuesta -->
            <div class="p-2 bg-obsidian-900 border-2 border-obsidian-750 rounded-xl flex items-center justify-center shadow-inner relative">
              <div class="relative w-56 h-56 rounded overflow-hidden">
                <!-- Canvas 2D de Pintura -->
                <canvas
                  bind:this={canvasEl}
                  width="64"
                  height="64"
                  onmousedown={handleCanvasMouseEvent}
                  onmousemove={handleCanvasMouseEvent}
                  onmouseup={handleCanvasMouseEvent}
                  onmouseleave={handleCanvasMouseEvent}
                  class="w-full h-full cursor-crosshair border border-obsidian-700 block"
                  style="image-rendering: pixelated; background: repeating-conic-gradient(#111420 0% 25%, #181c2e 0% 50%) 50% / 14px 14px;"
                ></canvas>

                <!-- Grilla de Píxeles 64x64 Vectorial/CSS -->
                {#if showGrid}
                  <div
                    class="absolute inset-0 pointer-events-none opacity-40"
                    style="background-size: calc(100% / 64) calc(100% / 64); background-image: linear-gradient(to right, rgba(255, 255, 255, 0.18) 1px, transparent 1px), linear-gradient(to bottom, rgba(255, 255, 255, 0.18) 1px, transparent 1px);"
                  ></div>
                {/if}

                <!-- Máscara visual de partes/capas bloqueadas -->
                <canvas
                  bind:this={overlayCanvasEl}
                  width="64"
                  height="64"
                  class="absolute inset-0 w-full h-full pointer-events-none"
                  style="image-rendering: pixelated;"
                ></canvas>
              </div>
            </div>
            <p class="text-[10px] text-slate-400 text-center">
              Pinta directamente aquí sobre las zonas visibles de la piel.
            </p>
          </div>

          <!-- Acciones de Almacenamiento & Despliegue -->
          <div class="space-y-2.5 pt-2 border-t border-obsidian-800">
            <button
              type="button"
              onclick={() => {
                if (!canvasEl) return;
                onSaveSkin({
                  id: `skin_${Date.now()}`,
                  name: skinName.trim() || 'Skin Sin Nombre',
                  model_type: modelType,
                  png_data_base64: canvasEl.toDataURL('image/png'),
                });
              }}
              class="w-full py-2 bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold rounded-xl transition flex items-center justify-center gap-2 text-xs shadow-md shadow-omni-indigo/25"
            >
              <Save class="w-4 h-4" /> Guardar en Biblioteca Local
            </button>

            <!-- Asignación al Perfil Seleccionado -->
            <button
              type="button"
              onclick={() => canvasEl && onInjectOffline(canvasEl.toDataURL('image/png'))}
              class="w-full py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-xl transition flex items-center justify-center gap-2 text-xs shadow-md shadow-emerald-600/25"
              title="Asigna esta skin al perfil seleccionado; se inyectará automáticamente al juego al arrancar"
            >
              <Sparkles class="w-4 h-4" /> Asignar al Perfil ({activeAccount?.username || 'Activo'})
            </button>

            <!-- Migración Mojang si cuenta Microsoft -->
            {#if activeAccount?.account_type === 'microsoft'}
              <button
                type="button"
                onclick={() => canvasEl && onUploadMojang(canvasEl.toDataURL('image/png'), modelType)}
                class="w-full py-2 bg-omni-cyan hover:bg-omni-cyan/80 text-obsidian-950 font-bold rounded-xl transition flex items-center justify-center gap-2 text-xs shadow-md shadow-omni-cyan/25"
              >
                <ArrowUpRight class="w-4 h-4" /> Subir a Mojang Oficial
              </button>
            {/if}

            <!-- Importar / Exportar PNG -->
            <div class="grid grid-cols-2 gap-2 pt-1">
              <label class="py-1.5 rounded-lg bg-obsidian-900 hover:bg-obsidian-800 border border-obsidian-750 text-white font-semibold text-center cursor-pointer transition flex items-center justify-center gap-1.5 text-xs">
                <Upload class="w-3.5 h-3.5 text-omni-cyan" /> Importar
                <input type="file" accept="image/png" onchange={handleImportPng} class="hidden" />
              </label>

              <button
                type="button"
                onclick={handleExportPng}
                class="py-1.5 rounded-lg bg-obsidian-900 hover:bg-obsidian-800 border border-obsidian-750 text-white font-semibold text-center transition flex items-center justify-center gap-1.5 text-xs"
              >
                <Download class="w-3.5 h-3.5 text-amber-400" /> Exportar
              </button>
            </div>
          </div>
        </div>
      </div>
      {/if}
    </div>
  </div>
{/if}
