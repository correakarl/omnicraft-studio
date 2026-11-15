<script lang="ts">
  import { X, Check, Shield, User, Globe, AlertCircle, RefreshCw, Key, LogOut, Trash2, ExternalLink } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { AccountModel, MicrosoftDeviceCodeModel, MicrosoftAccountSession } from '../types';
  import { SkinService } from '../skinService';

  let {
    isOpen = false,
    accounts = [] as AccountModel[],
    onClose = () => {},
    onSetAccount = (username: string, accountType: string, uuid?: string) => {},
    onDeleteAccount = (id: string) => {},
    onLogoutUser = (username: string) => {},
  } = $props<{
    isOpen: boolean;
    accounts: AccountModel[];
    onClose: () => void;
    onSetAccount: (username: string, accountType: string, uuid?: string) => void;
    onDeleteAccount: (id: string) => void;
    onLogoutUser: (username: string) => void;
  }>();

  let activeAccount = $derived(accounts.find(a => a.active) || accounts[0]);
  let newUsername = $state('');
  let accountType = $state<'offline' | 'microsoft'>('offline');
  let previewAvatar = $derived(SkinService.getAvatarUrl(newUsername || activeAccount?.username || 'Steve', 64));
  let previewBody = $derived(
    (!newUsername && activeAccount?.applied_skin_base64)
      ? activeAccount.applied_skin_base64
      : SkinService.getBodyUrl(newUsername || activeAccount?.username || 'Steve', 180)
  );

  // Estados de Microsoft OAuth Device Flow
  let msDeviceFlow = $state<MicrosoftDeviceCodeModel | null>(null);
  let isStartingMsAuth = $state(false);
  let isPollingMs = $state(false);
  let msPollingTimer: any = null;

  async function startMicrosoftLogin() {
    isStartingMsAuth = true;
    msDeviceFlow = null;
    try {
      const res = await invoke<MicrosoftDeviceCodeModel>('start_ms_auth_flow');
      msDeviceFlow = res;
      pollToken(res.device_code, res.interval || 5);
    } catch (e) {
      alert(`Error al iniciar flujo Microsoft: ${e}`);
    } finally {
      isStartingMsAuth = false;
    }
  }

  function pollToken(deviceCode: string, intervalSecs: number) {
    if (msPollingTimer) clearInterval(msPollingTimer);
    isPollingMs = true;

    msPollingTimer = setInterval(async () => {
      try {
        const session = await invoke<MicrosoftAccountSession>('poll_ms_auth_token', { deviceCode });
        if (session && session.username) {
          clearInterval(msPollingTimer);
          isPollingMs = false;
          // Usuario autenticado exitosamente con datos oficiales
          onSetAccount(session.username, 'microsoft', session.uuid);
          msDeviceFlow = null;
          alert(`✅ ¡Cuenta oficial vinculada con éxito! Bienvenido, ${session.username}.`);
        }
      } catch (err: any) {
        if (typeof err === 'string' && (err === 'pending' || err.includes('pending'))) {
          // Aún esperando confirmación del usuario
          return;
        }
        clearInterval(msPollingTimer);
        isPollingMs = false;
        alert(`Aviso de inicio de sesión: ${err}`);
      }
    }, Math.max(intervalSecs, 5) * 1000);
  }

  function handleCreateOffline() {
    if (!newUsername.trim()) return;
    onSetAccount(newUsername.trim(), 'offline');
    newUsername = '';
  }

  function handleSelectExisting(acc: AccountModel) {
    onSetAccount(acc.username, acc.account_type, acc.uuid || undefined);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="bg-obsidian-900 border border-obsidian-750/90 rounded-2xl w-full max-w-xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-obsidian-750 flex items-center justify-between bg-obsidian-900/90">
        <div>
          <h2 class="text-base font-bold text-white flex items-center gap-2">
            <Shield class="w-5 h-5 text-omni-cyan" />
            Gestión de Cuentas & Previsualizador de Skins
          </h2>
          <p class="text-xs text-slate-400 mt-0.5">
            Cuentas Non-Premium (Offline) y Microsoft Oficial con renderizado de Skin 2D/3D.
          </p>
        </div>
        <button onclick={onClose} class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-obsidian-800 transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 overflow-y-auto space-y-6 text-xs text-slate-200">
        <!-- Renderizador de Skin y Jugador Activo -->
        <div class="p-4 rounded-xl bg-obsidian-950/80 border border-obsidian-800 flex items-center gap-5">
          <div class="w-20 h-28 bg-obsidian-900 rounded-lg flex items-center justify-center p-1 border border-obsidian-750">
            <img
              src={previewBody}
              alt="Player Skin"
              class="max-h-full object-contain filter drop-shadow"
              onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/minecraft-icons/mini-steve.png'; }}
            />
          </div>

          <div class="flex-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-bold text-white">{activeAccount?.username || 'Sin cuenta'}</span>
              <span class="px-2 py-0.5 rounded text-[10px] font-mono {activeAccount?.account_type === 'microsoft' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-slate-700 text-slate-300'}">
                {activeAccount?.account_type === 'microsoft' ? 'Microsoft Oficial' : 'Non-Premium (Offline)'}
              </span>
            </div>
            <p class="text-[11px] text-slate-400 mt-1">
              {activeAccount?.uuid ? `UUID: ${activeAccount.uuid.slice(0, 16)}...` : 'Identidad local autónoma para partidas individuales y servidores compatibles.'}
            </p>
            <p class="text-[10px] text-omni-cyan mt-2 font-medium">
              Renderizado directo con Mojang Textures / mc-heads CDN.
            </p>
            {#if activeAccount}
              <div class="mt-3 pt-2.5 border-t border-obsidian-800 flex items-center gap-2">
                <button
                  type="button"
                  onclick={() => onLogoutUser(activeAccount.username)}
                  class="px-2.5 py-1 rounded bg-rose-500/20 hover:bg-rose-500/30 text-rose-400 font-bold text-[11px] border border-rose-500/30 transition flex items-center gap-1.5"
                  title="Cerrar sesión y limpiar registros locales de esta cuenta"
                >
                  <LogOut class="w-3.5 h-3.5" /> Cerrar Sesión y Limpiar
                </button>
              </div>
            {/if}
          </div>
        </div>

        <!-- Cuentas Registradas -->
        <div>
          <h3 class="text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">Cuentas Guardadas</h3>
          <div class="space-y-1.5 max-h-40 overflow-y-auto">
            {#each accounts as acc}
              <div
                class="w-full p-2 rounded-lg border text-left flex items-center justify-between transition {acc.active ? 'bg-omni-indigo/20 border-omni-indigo text-white font-bold' : 'bg-obsidian-950/50 border-obsidian-800 text-slate-300 hover:border-obsidian-700'}"
              >
                <button
                  type="button"
                  onclick={() => handleSelectExisting(acc)}
                  class="flex items-center gap-2.5 flex-1 min-w-0 text-left"
                >
                  <img src={SkinService.getAvatarUrl(acc.username, 24)} alt="" class="w-6 h-6 rounded" />
                  <span class="truncate">{acc.username}</span>
                  <span class="text-[10px] text-slate-400 font-normal">({acc.account_type})</span>
                </button>

                <div class="flex items-center gap-2">
                  {#if acc.active}
                    <span class="px-2 py-0.5 rounded text-[10px] bg-emerald-500/20 text-emerald-400 font-mono">Activa</span>
                  {/if}
                  <button
                    type="button"
                    onclick={(e) => { e.stopPropagation(); onDeleteAccount(acc.id); }}
                    class="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition"
                    title="Eliminar cuenta y registros de Omnicraft"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Añadir Nueva Cuenta -->
        <div class="p-4 bg-obsidian-950/60 border border-obsidian-800 rounded-xl space-y-3">
          <h3 class="text-xs font-bold text-slate-200">Añadir o Cambiar Jugador</h3>
          <div class="flex gap-2">
            <button
              type="button"
              onclick={() => accountType = 'offline'}
              class="flex-1 py-1.5 rounded-lg border text-center font-medium {accountType === 'offline' ? 'bg-omni-indigo text-white border-omni-indigo' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
            >
              Non-Premium (Offline)
            </button>
            <button
              type="button"
              onclick={() => accountType = 'microsoft'}
              class="flex-1 py-1.5 rounded-lg border text-center font-medium {accountType === 'microsoft' ? 'bg-emerald-600 text-white border-emerald-500' : 'bg-obsidian-900 border-obsidian-750 text-slate-400'}"
            >
              Microsoft Oficial
            </button>
          </div>

          {#if accountType === 'offline'}
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={newUsername}
                placeholder="Nombre de usuario (ej: Notch o Alex)"
                class="flex-1 bg-obsidian-900 border border-obsidian-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-omni-cyan"
              />
              <button
                type="button"
                onclick={handleCreateOffline}
                class="px-4 py-2 bg-omni-indigo hover:bg-omni-indigo/80 text-white font-bold rounded-lg transition"
              >
                Guardar
              </button>
            </div>
          {:else}
            <div class="p-3.5 bg-obsidian-900 rounded-lg border border-obsidian-750 space-y-3">
              <p class="text-[11px] text-slate-300">
                Flujo OAuth2 Device Code: no requiere ingresar contraseña en el lanzador. Abre el enlace oficial de Microsoft e introduce el código temporal.
              </p>

              {#if msDeviceFlow}
                <div class="p-3 bg-obsidian-950 rounded-lg border border-emerald-500/30 space-y-2">
                  <div class="flex items-center justify-between text-xs font-bold text-white">
                    <span>Código de Vinculación:</span>
                    <span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-mono text-sm tracking-widest select-all">
                      {msDeviceFlow.user_code}
                    </span>
                  </div>
                  <p class="text-[11px] text-slate-400">
                    Ingresa en <a href={msDeviceFlow.verification_uri} target="_blank" class="text-omni-cyan underline font-bold">microsoft.com/link</a> e introduce el código de arriba.
                  </p>
                  {#if isPollingMs}
                    <div class="flex items-center gap-2 text-[10px] text-amber-300 font-mono pt-1">
                      <RefreshCw class="w-3 h-3 animate-spin text-omni-cyan" />
                      <span>Esperando confirmación en Microsoft...</span>
                    </div>
                  {/if}
                </div>
              {:else}
                <button
                  type="button"
                  onclick={startMicrosoftLogin}
                  disabled={isStartingMsAuth}
                  class="w-full py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg transition flex items-center justify-center gap-2 text-xs shadow-md shadow-emerald-600/25 disabled:opacity-50"
                >
                  {#if isStartingMsAuth}
                    <RefreshCw class="w-4 h-4 animate-spin" /> Conectando con Microsoft...
                  {:else}
                    <Key class="w-4 h-4" /> Generar Código de Inicio de Sesión
                  {/if}
                </button>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-obsidian-750 bg-obsidian-900/90 flex justify-end">
        <button
          type="button"
          onclick={onClose}
          class="px-5 py-2 rounded-xl text-xs font-bold bg-obsidian-800 hover:bg-obsidian-700 text-white transition"
        >
          Cerrar
        </button>
      </div>
    </div>
  </div>
{/if}
