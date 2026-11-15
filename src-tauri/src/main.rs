// Omnicraft Launcher Studio v2.0 - Entrypoint de Escritorio
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use omnicraft_v2_lib::{
    get_core_status, DbManager, InstanceModel, AccountModel, RecommendationModel, ModItemModel,
    JavaService, JavaRuntime, LaunchEngine, ProcessInfo, ProfileSyncService,
    ModrinthClient, ModSearchResult, ModrinthDownloadService,
    CurseForgeService, UnifiedSearchResult, MojangManifestService, GameVersionEntry,
    LocalSkinModel, SkinBridgeService, RemotePublicSkin,
    ContentToolsService, WorldSaveInfo, LogOutput,
    ContentManagerService, ContentItem,
    CrashParserService, CrashDiagnosis, ServerBuilderService,
    MalwareScannerService, ScanResult, UpdateCheckerService, AppUpdateInfo,
    LoaderInstallerService, JreProvisionerService, MicrosoftAuthService, MicrosoftDeviceCodeResponse, MicrosoftAccountSession,
    ModpackInstallerService, CompatibilityCheckerService, CompatibilityReport,
    SharedStorageService, SysMonitor, InstanceResourceUsage
};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State,
};
use std::path::PathBuf;

struct AppState {
    db: DbManager,
    engine: LaunchEngine,
}

#[tauri::command]
fn get_instances(state: State<AppState>) -> Result<Vec<InstanceModel>, String> {
    state.db.get_all_instances().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_accounts(state: State<AppState>) -> Result<Vec<AccountModel>, String> {
    state.db.get_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_official_recs(category: String, state: State<AppState>) -> Result<Vec<RecommendationModel>, String> {
    state.db.get_official_recommendations(&category).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_instance_mods(instance_id: String, state: State<AppState>) -> Result<Vec<ModItemModel>, String> {
    state.db.get_instance_mods(&instance_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn detect_javas() -> Vec<JavaRuntime> {
    JavaService::detect_all_javas()
}

#[tauri::command]
fn get_running_instances(state: State<AppState>) -> Vec<ProcessInfo> {
    state.engine.get_active_instances()
}

#[tauri::command]
fn get_instance_resource_usage(instance_id: String, state: State<AppState>) -> Option<InstanceResourceUsage> {
    let procs = state.engine.get_active_instances();
    if let Some(info) = procs.into_iter().find(|p| p.instance_id == instance_id) {
        Some(SysMonitor::get_process_usage(
            info.pid,
            Some(info.instance_id),
            Some(info.instance_name),
            Some(info.username),
        ))
    } else {
        None
    }
}

#[tauri::command]
fn get_all_instances_resource_usage(state: State<AppState>) -> Vec<InstanceResourceUsage> {
    let procs = state.engine.get_active_instances();
    procs.into_iter().map(|p| {
        SysMonitor::get_process_usage(
            p.pid,
            Some(p.instance_id),
            Some(p.instance_name),
            Some(p.username),
        )
    }).collect()
}

#[tauri::command]
fn terminate_instance(instance_id: String, state: State<AppState>) -> bool {
    state.engine.terminate(&instance_id)
}

#[tauri::command]
fn send_server_cmd(instance_id: String, command: String, state: State<AppState>) -> Result<(), String> {
    state.engine.send_server_command(&instance_id, &command)
}

#[tauri::command]
async fn search_store_unified(
    source: String, // "all" | "modrinth" | "curseforge"
    query: String,
    project_type: String,
    mc_version: String,
    loader: String,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<UnifiedSearchResult>, String> {
    let cf_key = state.db.get_setting("curseforge_api_key").ok().flatten();
    let mut results = Vec::new();

    let run_modrinth = source == "all" || source == "modrinth";
    let run_curseforge = source == "all" || source == "curseforge";

    if run_modrinth {
        if let Ok(m_hits) = ModrinthClient::search(&query, &project_type, &mc_version, &loader, limit).await {
            for h in m_hits {
                let ptype = h.project_type.clone();
                let web = format!("https://modrinth.com/{}/{}", ptype, h.slug);
                results.push(UnifiedSearchResult {
                    id: h.id,
                    source: "Modrinth".to_string(),
                    slug: h.slug,
                    title: h.title,
                    description: h.description,
                    author: h.author,
                    downloads: h.downloads,
                    icon_url: h.icon_url,
                    project_type: ptype,
                    web_url: web,
                    loaders: h.loaders,
                    versions: h.versions,
                });
            }
        }
    }

    if run_curseforge {
        if let Ok(cf_hits) = CurseForgeService::search(
            &query,
            &project_type,
            &mc_version,
            &loader,
            limit,
            cf_key.as_deref(),
        ).await {
            results.extend(cf_hits);
        }
    }

    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    Ok(results)
}

#[tauri::command]
async fn get_all_game_versions(state: State<'_, AppState>) -> Result<Vec<GameVersionEntry>, String> {
    // 1. Intentar leer de base de datos SQLite local
    if let Ok(cached) = state.db.get_cached_versions() {
        if !cached.is_empty() {
            let res = cached
                .into_iter()
                .map(|(id, version_type)| GameVersionEntry {
                    id,
                    version_type,
                    release_time: None,
                })
                .collect();
            return Ok(res);
        }
    }

    // 2. Si no hay caché, consultar directamente el manifiesto de Mojang y cachear
    let list = MojangManifestService::fetch_all_versions().await?;
    let to_save: Vec<(String, String, Option<String>)> = list
        .iter()
        .map(|e| (e.id.clone(), e.version_type.clone(), e.release_time.clone()))
        .collect();
    state.db.save_cached_versions(&to_save).ok();

    Ok(list)
}

#[tauri::command]
async fn refresh_game_versions(state: State<'_, AppState>) -> Result<Vec<GameVersionEntry>, String> {
    let list = MojangManifestService::fetch_all_versions().await?;
    let to_save: Vec<(String, String, Option<String>)> = list
        .iter()
        .map(|e| (e.id.clone(), e.version_type.clone(), e.release_time.clone()))
        .collect();
    state.db.save_cached_versions(&to_save).ok();
    Ok(list)
}

#[tauri::command]
fn get_local_skins(state: State<AppState>) -> Result<Vec<LocalSkinModel>, String> {
    state.db.get_all_local_skins().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_profile_skin_cmd(
    username: String,
    uuid: Option<String>,
    account_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<(String, String)>, String> {
    // 1. Verificar si el perfil tiene una skin aplicada directamente en la BD
    let key_setting = format!("user_skin_b64:{}", username.to_lowercase());
    if let Ok(Some(b64)) = state.db.get_setting(&key_setting) {
        if !b64.is_empty() {
            return Ok(Some((b64, "classic".to_string())));
        }
    }

    if let Ok(accounts) = state.db.get_accounts() {
        if let Some(acc) = accounts.iter().find(|a| a.username.eq_ignore_ascii_case(&username)) {
            if let Some(ref b64) = acc.applied_skin_base64 {
                if !b64.is_empty() {
                    return Ok(Some((b64.clone(), "classic".to_string())));
                }
            }
        }
    }

    // 2. Si es cuenta Microsoft o tiene skin_url oficial guardada en credenciales, descargarla
    if let Some((_, _, _, Some(s_url))) = state.db.get_account_full_credentials(&username) {
        if !s_url.is_empty() {
            if let Ok(b64) = SkinBridgeService::download_remote_skin_base64(&s_url).await {
                return Ok(Some((b64, "classic".to_string())));
            }
        }
    }

    // 3. Si es cuenta Microsoft (o si el username existe en Mojang), consultar Mojang API
    let is_msa = account_type.as_deref() == Some("microsoft");
    if is_msa {
        let query_id = uuid.as_deref().filter(|u| !u.is_empty()).unwrap_or(&username);
        if let Ok((b64, model)) = SkinBridgeService::fetch_mojang_skin(query_id).await {
            return Ok(Some((b64, model)));
        }
    }

    // 4. Usar la resolución general de skin aplicada al perfil o sesión activa (o biblioteca local)
    if let Some(b64) = state.db.get_user_applied_skin(&username) {
        return Ok(Some((b64, "classic".to_string())));
    }

    // 5. Por último, si la sesión activa de Microsoft tiene skin_url, usarla
    if let Some(active_url) = state.db.get_active_session_skin_url() {
        if let Ok(b64) = SkinBridgeService::download_remote_skin_base64(&active_url).await {
            return Ok(Some((b64, "classic".to_string())));
        }
    }

    Ok(None)
}

#[tauri::command]
fn save_local_skin(
    id: String,
    name: String,
    model_type: String,
    base64_png: String,
    state: State<AppState>,
) -> Result<(), String> {
    state.db.save_local_skin(&id, &name, &model_type, &base64_png).map_err(|e| e.to_string())?;
    // Si aún no hay ninguna skin aplicada para la cuenta de la sesión activa, asignarla automáticamente
    if let Ok(accounts) = state.db.get_accounts() {
        if let Some(act) = accounts.iter().find(|a| a.active) {
            if act.applied_skin_base64.is_none() {
                let _ = state.db.set_user_applied_skin(&act.username, Some(&id), &base64_png);
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn delete_local_skin(id: String, state: State<AppState>) -> Result<(), String> {
    state.db.delete_local_skin(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn assign_skin_to_profile_cmd(
    username: String,
    skin_id: Option<String>,
    base64_png: String,
    state: State<AppState>,
) -> Result<(), String> {
    // Solo asigna la skin al perfil en la base de datos (sin tocar archivos de instancias).
    // La inyección física al juego se ejecuta únicamente al arrancar la instancia desde el perfil seleccionado.
    state.db.set_user_applied_skin(&username, skin_id.as_deref(), &base64_png)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn inject_offline_skin_to_instance(
    folder_name: String,
    username: String,
    base64_png: String,
    state: State<AppState>,
) -> Result<(), String> {
    let _ = folder_name;
    state.db.set_user_applied_skin(&username, None, &base64_png)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn upload_skin_to_mojang(
    username: String,
    bearer_token: Option<String>,
    base64_png: String,
    model_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Si viene token directo se utiliza, de lo contrario se obtiene de la bóveda segura del usuario activo
    let token = if let Some(t) = bearer_token.filter(|t| !t.is_empty()) {
        t
    } else {
        let creds = state.db.get_active_account_credentials(&username);
        creds.map(|c| c.1).ok_or_else(|| "No se encontraron credenciales válidas de Microsoft/Mojang para este usuario. Inicia sesión en la cuenta oficial primero.".to_string())?
    };

    SkinBridgeService::upload_skin_to_mojang(&token, &base64_png, &model_type).await?;
    // También registrar la skin como la skin activa del usuario en la base de datos local para inyección automática
    let _ = state.db.set_user_applied_skin(&username, None, &base64_png);
    Ok(())
}

#[tauri::command]
async fn fetch_mojang_skin_cmd(
    username_or_uuid: String,
) -> Result<(String, String), String> {
    SkinBridgeService::fetch_mojang_skin(&username_or_uuid).await
}

#[tauri::command]
async fn search_remote_skins_cmd(
    query: String,
    page: u32,
) -> Result<Vec<RemotePublicSkin>, String> {
    SkinBridgeService::search_remote_public_skins(&query, page).await
}

#[tauri::command]
async fn download_remote_skin_cmd(
    texture_url: String,
) -> Result<String, String> {
    SkinBridgeService::download_remote_skin_base64(&texture_url).await
}

#[tauri::command]
fn get_api_key(service: String, state: State<AppState>) -> Result<Option<String>, String> {
    let key_name = format!("{}_api_key", service);
    state.db.get_setting(&key_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_api_key(service: String, key_value: String, state: State<AppState>) -> Result<(), String> {
    let key_name = format!("{}_api_key", service);
    state.db.set_setting(&key_name, &key_value).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_modrinth(
    query: String,
    project_type: String,
    mc_version: String,
    loader: String,
    limit: u32,
) -> Result<Vec<ModSearchResult>, String> {
    ModrinthClient::search(&query, &project_type, &mc_version, &loader, limit).await
}

#[tauri::command]
fn parse_crash_log(log_text: String) -> CrashDiagnosis {
    CrashParserService::parse(&log_text)
}

#[tauri::command]
fn setup_server_instance(instance_dir: String, port: u16) -> Result<(), String> {
    let path = PathBuf::from(instance_dir);
    ServerBuilderService::setup_server(&path, port)
}

#[tauri::command]
fn sync_instance_profile(
    instance_name: String,
    mc_dir: String,
    mc_version: String,
    loader_type: String,
    java_bin: String,
    ram_min_mb: u32,
    ram_max_mb: u32,
    jvm_preset: Option<String>,
    jvm_args: String,
    width: u32,
    height: u32,
    username: String,
    uuid: Option<String>,
    access_token: Option<String>,
) -> Result<(), String> {
    let path = PathBuf::from(mc_dir);
    ProfileSyncService::sync_instance_profile(
        &instance_name,
        &path,
        &mc_version,
        &loader_type,
        &java_bin,
        ram_min_mb,
        ram_max_mb,
        jvm_preset.as_deref(),
        &jvm_args,
        width,
        height,
        &username,
        uuid.as_deref(),
        access_token.as_deref(),
        None,
    )
}

#[tauri::command]
fn get_default_instance_id(state: State<AppState>) -> Option<String> {
    state.db.get_setting("default_instance_id").ok().flatten()
}

#[tauri::command]
fn set_default_instance_id(instance_id: String, state: State<AppState>) -> Result<(), String> {
    state.db.set_setting("default_instance_id", &instance_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_instance_cmd(
    instance_id: String,
    username: String,
    uuid: Option<String>,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let instances = state.db.get_all_instances().map_err(|e| e.to_string())?;
    let inst = instances
        .iter()
        .find(|i| i.id == instance_id)
        .cloned()
        .ok_or_else(|| format!("Instancia '{}' no encontrada", instance_id))?;

    // Determinar cuál es la Instancia Principal (Default)
    let saved_default_id = state.db.get_setting("default_instance_id").ok().flatten();
    let default_inst_id = saved_default_id
        .filter(|did| instances.iter().any(|i| &i.id == did))
        .or_else(|| instances.first().map(|i| i.id.clone()))
        .unwrap_or_else(|| inst.id.clone());

    let is_default_instance = inst.id == default_inst_id;

    let accounts = state.db.get_accounts().unwrap_or_default();
    let active_account = accounts.iter().find(|a| a.active).or_else(|| accounts.first());

    let auto_placeholder = format!("Player_{}", inst.name.replace(' ', "_"));
    let has_custom_assigned = inst.assigned_username.as_ref()
        .map(|u| {
            let trimmed = u.trim();
            !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case(&auto_placeholder)
        })
        .unwrap_or(false);

    // REGLA DE SINCRONIZACIÓN DE CUENTAS E INSTANCIAS:
    // 1. La Instancia por Default arranca en sincronía con la Cuenta Principal por Default (salvo que tenga una cuenta offline asignada explícitamente).
    // 2. La Cuenta Oficial Microsoft SOLO puede iniciar sobre la Instancia por Default seleccionada.
    // 3. Las demás instancias secundarias utilizan automáticamente cuentas Non-Premium (su cuenta asignada, o una cuenta Non-Premium sin seleccionar de la lista, o Player_<Instancia>).
    let (effective_username, matched_account) = if is_default_instance {
        if has_custom_assigned {
            let custom_u = inst.assigned_username.clone().unwrap();
            let acc = accounts.iter().find(|a| a.username.eq_ignore_ascii_case(&custom_u)).cloned();
            (custom_u, acc)
        } else if let Some(act) = active_account {
            (act.username.clone(), Some(act.clone()))
        } else if !username.trim().is_empty() {
            let acc = accounts.iter().find(|a| a.username.eq_ignore_ascii_case(&username)).cloned();
            (username, acc)
        } else {
            (auto_placeholder, None)
        }
    } else {
        // Instancia secundaria (No-Default): SOLO permite cuentas Non-Premium (Offline)
        if has_custom_assigned {
            let custom_u = inst.assigned_username.clone().unwrap();
            let acc = accounts.iter().find(|a| a.username.eq_ignore_ascii_case(&custom_u)).cloned();
            // Si el nombre asignado corresponde a una cuenta Microsoft, redirigir a cuenta offline
            if acc.as_ref().map(|a| a.account_type == "microsoft").unwrap_or(false) {
                let fallback_offline = accounts.iter().find(|a| a.account_type != "microsoft" && !a.active)
                    .or_else(|| accounts.iter().find(|a| a.account_type != "microsoft"))
                    .cloned();
                if let Some(off) = fallback_offline {
                    (off.username.clone(), Some(off))
                } else {
                    (auto_placeholder, None)
                }
            } else {
                (custom_u, acc)
            }
        } else {
            // Buscar primero una cuenta Non-Premium sin seleccionar (active == false)
            let unselected_offline = accounts.iter()
                .find(|a| a.account_type != "microsoft" && !a.active)
                .or_else(|| accounts.iter().find(|a| a.account_type != "microsoft"))
                .cloned();
            if let Some(off) = unselected_offline {
                (off.username.clone(), Some(off))
            } else {
                (auto_placeholder, None)
            }
        }
    };

    let is_msa = matched_account.as_ref().map(|a| a.account_type == "microsoft").unwrap_or(false);

    // Doble verificación estricta: Cuenta Microsoft solo en instancia Default y una sola instancia a la vez
    if is_msa && !is_default_instance {
        return Err("La cuenta oficial Microsoft solo puede iniciar sobre la instancia seleccionada por Default. Establece esta instancia como Default o usa una cuenta Non-Premium.".to_string());
    }
    if is_msa && state.engine.is_msa_in_use() {
        return Err("La cuenta oficial Microsoft ya se encuentra en ejecución en la instancia principal. Las instancias adicionales deben ejecutarse con cuentas Non-Premium.".to_string());
    }

    let user_type = if is_msa { "msa" } else { "legacy" };

    // Si es Microsoft, verificar validez del token y renovarlo con refresh_token si expiró
    let (resolved_uuid, resolved_token, mut resolved_skin_url) = if is_msa {
        if let Some((mut u, mut tok, ref_tok, mut skin_url)) = state.db.get_account_full_credentials(&effective_username) {
            if !MicrosoftAuthService::is_minecraft_token_valid(&tok).await {
                if let Some(rt) = ref_tok.as_deref() {
                    if let Ok(refreshed) = MicrosoftAuthService::refresh_microsoft_session(rt).await {
                        let _ = state.db.save_microsoft_account(
                            &refreshed.username,
                            &refreshed.uuid,
                            &refreshed.access_token,
                            refreshed.refresh_token.as_deref(),
                            refreshed.skin_url.as_deref(),
                        );
                        u = refreshed.uuid;
                        tok = refreshed.access_token;
                        skin_url = refreshed.skin_url;
                    }
                }
            }
            (Some(u), Some(tok), skin_url)
        } else {
            (matched_account.as_ref().and_then(|a| a.uuid.clone()), None, None)
        }
    } else {
        // Para cuentas Non-Premium, heredar también la skin_url de la sesión activa si existe
        let active_skin_url = state.db.get_active_session_skin_url();
        (matched_account.as_ref().and_then(|a| a.uuid.clone()), None, active_skin_url)
    };

    let player_uuid = resolved_uuid.as_deref().or(uuid.as_deref());

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut root_path = PathBuf::from(&home);
    root_path.push(".omnicraft_studio");
    root_path.push("instances");
    root_path.push(&inst.folder);

    let mut mc_path = root_path.clone();
    mc_path.push(".minecraft");

    // INYECCIÓN AUTOMÁTICA DE SKIN DEL USERNAME O DE LA SESIÓN ACTIVA:
    // Antes de lanzar cualquier instancia cliente, si el username efectivo o la sesión activa
    // tiene una skin aplicada en la BD (o si la sesión activa de Microsoft tiene su skin oficial),
    // se inyecta automáticamente en CustomSkinLoader y SkinShuffle para ese username.
    if inst.instance_type == "client" {
        if let Some(user_b64) = state.db.get_user_applied_skin(&effective_username) {
            let _ = SkinBridgeService::inject_offline_skin(&mc_path, &effective_username, &user_b64);
        } else if let Some(ref s_url) = resolved_skin_url {
            if let Ok(dl_b64) = SkinBridgeService::download_remote_skin_base64(s_url).await {
                let _ = SkinBridgeService::inject_offline_skin(&mc_path, &effective_username, &dl_b64);
            }
        }
        // Si el usuario no tenía skin_url pero existe un perfil en Mojang para la cuenta activa, intentar obtenerlo
        if resolved_skin_url.is_none() {
            if let Some(act) = active_account {
                if let Some((_, _, _, s_url)) = state.db.get_account_full_credentials(&act.username) {
                    resolved_skin_url = s_url;
                }
            }
        }
    }

    // Resolver Java: seleccionar la versión idónea según la versión de Minecraft o configuración
    let javas = JavaService::detect_all_javas();
    let java_bin = if inst.java_selection_mode == "custom" && inst.custom_java_path.is_some() {
        inst.custom_java_path.clone().unwrap()
    } else {
        let preferred_version: u32 = if inst.mc_version.starts_with("1.16") || inst.mc_version.starts_with("1.12") || inst.mc_version.starts_with("1.8") || inst.mc_version.starts_with("1.7") {
            8
        } else if inst.mc_version.starts_with("1.17") || inst.mc_version.starts_with("1.18") || inst.mc_version.starts_with("1.19") || inst.mc_version.starts_with("1.20.1") || inst.mc_version.starts_with("1.20.2") || inst.mc_version.starts_with("1.20.4") {
            17
        } else {
            21 // MC 1.20.5+ y launchers modernos
        };

        javas.iter()
            .find(|j| j.version == preferred_version)
            .or_else(|| javas.iter().find(|j| j.version >= 17))
            .or_else(|| javas.first())
            .map(|j| j.path.clone())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let local_java = PathBuf::from(&home).join(".local/bin/java");
                if local_java.exists() {
                    local_java.to_string_lossy().to_string()
                } else {
                    "java".to_string()
                }
            })
    };

    // Sincronizar perfiles técnicos en launcher_profiles.json, .hmcl y SkinShuffle (Java, RAM, loader, MC version, cuenta y skin)
    if inst.instance_type == "client" {
        ProfileSyncService::sync_instance_profile(
            &inst.name,
            &mc_path,
            &inst.mc_version,
            &inst.loader_type,
            &java_bin,
            inst.ram_min_mb,
            inst.ram_max_mb,
            inst.jvm_preset.as_deref(),
            inst.jvm_args.as_deref().unwrap_or(""),
            inst.width,
            inst.height,
            &effective_username,
            player_uuid,
            resolved_token.as_deref(),
            resolved_skin_url.as_deref(),
        )?;
    } else {
        // Preparar servidor si aplica
        ServerBuilderService::setup_server(&mc_path, inst.server_port)?;
    }

    state.engine.launch_instance(
        &inst.id,
        &inst.name,
        &inst.instance_type,
        &inst.mc_version,
        &inst.loader_type,
        &mc_path,
        &root_path,
        &java_bin,
        inst.ram_min_mb,
        inst.ram_max_mb,
        inst.jvm_preset.as_deref(),
        inst.jvm_args.as_deref(),
        &inst.executable_type,
        inst.launcher_jar.as_deref(),
        inst.custom_executable_path.as_deref(),
        inst.server_port,
        &effective_username,
        player_uuid,
        resolved_token.as_deref(),
        user_type,
        inst.width,
        inst.height,
    )
}

#[tauri::command]
async fn install_mod_to_instance(
    app: tauri::AppHandle,
    source: String, // "Modrinth" | "CurseForge"
    project_id: String,
    instance_id: String,
    folder_name: String,
    mc_version: String,
    loader: String,
    project_type: String,
    mod_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut mc_dir = PathBuf::from(&home);
    mc_dir.push(".omnicraft_studio");
    mc_dir.push("instances");
    mc_dir.push(&folder_name);
    mc_dir.push(".minecraft");

    let (subfolder, is_mod, is_modpack) = match project_type.as_str() {
        "shader" => ("shaderpacks", false, false),
        "resourcepack" => ("resourcepacks", false, false),
        "modpack" => ("modpacks", false, true),
        _ => ("mods", true, false),
    };

    let dest = mc_dir.join(subfolder);
    std::fs::create_dir_all(&dest).ok();

    // Notificar inicio de descarga en la cola de descargas
    let task_id = format!("{}-{}", project_id, chrono::Utc::now().timestamp_millis());
    let _ = app.emit("download-event", serde_json::json!({
        "id": task_id,
        "title": mod_name,
        "source": source,
        "status": "Descargando paquete...",
        "progress": 15.0,
        "completed": false,
    }));

    let filename = if source == "CurseForge" {
        let cf_key = state.db.get_setting("curseforge_api_key").ok().flatten();
        CurseForgeService::download_item(
            &project_id,
            &mc_version,
            &loader,
            &dest,
            subfolder,
            cf_key.as_deref(),
        ).await?
    } else {
        ModrinthDownloadService::download_project(
            &project_id,
            &mc_version,
            &loader,
            &dest,
            subfolder,
        ).await?
    };

    // Si es un modpack (.mrpack de Modrinth o .zip de CurseForge), desempaquetar configs y descargar todos los mods automáticamente
    if is_modpack && (filename.ends_with(".mrpack") || filename.ends_with(".zip")) {
        let pack_path = dest.join(&filename);
        let app_handle = app.clone();
        let tid = task_id.clone();
        let src_name = source.clone();
        let cf_key = state.db.get_setting("curseforge_api_key").ok().flatten();

        let pack_name = ModpackInstallerService::install_modpack_auto(&pack_path, &mc_dir, cf_key.as_deref(), move |p| {
            let _ = app_handle.emit("download-event", serde_json::json!({
                "id": tid,
                "title": format!("Instalando Modpack: {}", p.current_file),
                "source": src_name,
                "status": p.step,
                "progress": p.percent,
                "completed": p.percent >= 100.0,
            }));
        }).await?;

        // Sincronizar todos los mods descomprimidos en la BD de la instancia
        let mods_path = mc_dir.join("mods");
        if let Ok(entries) = std::fs::read_dir(&mods_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let fn_str = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let sz = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                    state.db.register_mod(
                        &instance_id,
                        &fn_str,
                        None,
                        &fn_str.replace(".jar", ""),
                        None,
                        Some(&loader),
                        sz,
                    ).ok();
                }
            }
        }

        let _ = app.emit("download-event", serde_json::json!({
            "id": task_id,
            "title": format!("Modpack {} Instalado", pack_name),
            "source": source,
            "status": "Completado y verificado",
            "progress": 100.0,
            "completed": true,
        }));

        return Ok(format!("Modpack {} extraído con éxito (mods y configuraciones listos)", pack_name));
    }

    // Si es un mod ejecutable/jar, deduplicar versiones anteriores del mismo mod en la carpeta
    if is_mod {
        let file_path = dest.join(&filename);
        let size_bytes = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        // Deduplicación preventiva en disco: si ya existía una versión anterior del mismo mod,
        // eliminar el .jar anterior para evitar colisiones ClassNotFound / NullPointer / MixinConflict
        let current_base = filename.split('-').next().unwrap_or(&filename).to_lowercase();
        if let Ok(entries) = std::fs::read_dir(&dest) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p != file_path && p.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let fn_other = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let other_base = fn_other.split('-').next().unwrap_or(&fn_other).to_lowercase();
                    if other_base == current_base {
                        std::fs::remove_file(&p).ok();
                        state.db.unregister_mod(&instance_id, &fn_other).ok();
                    }
                }
            }
        }

        state.db.register_mod(
            &instance_id,
            &filename,
            Some(&project_id),
            &mod_name,
            None,
            Some(&loader),
            size_bytes,
        ).ok();
    }

    let _ = app.emit("download-event", serde_json::json!({
        "id": task_id,
        "title": mod_name,
        "source": source,
        "status": "Instalado con éxito",
        "progress": 100.0,
        "completed": true,
    }));

    Ok(filename)
}

#[tauri::command]
fn check_instance_compatibility(
    folder_name: String,
    mc_version: String,
    loader: String,
) -> CompatibilityReport {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut mc_dir = PathBuf::from(home);
    mc_dir.push(".omnicraft_studio");
    mc_dir.push("instances");
    mc_dir.push(folder_name);
    mc_dir.push(".minecraft");

    CompatibilityCheckerService::check_instance_mods(&mc_dir, &mc_version, &loader)
}

#[derive(serde::Serialize)]
struct SharedCacheItemInfo {
    filename: String,
    category: String,
    size_bytes: u64,
    is_linked_to_instance: bool,
}

#[tauri::command]
fn list_shared_cache_items(
    folder_name: String,
    category: String,
) -> Vec<SharedCacheItemInfo> {
    let cache_dir = SharedStorageService::get_category_cache_dir(&category);
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let instance_cat_dir = PathBuf::from(home)
        .join(".omnicraft_studio")
        .join("instances")
        .join(folder_name)
        .join(".minecraft")
        .join(&category);

    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let linked = instance_cat_dir.join(&fname).exists() || instance_cat_dir.join(format!("{}.disabled", fname)).exists();

                list.push(SharedCacheItemInfo {
                    filename: fname,
                    category: category.clone(),
                    size_bytes: size,
                    is_linked_to_instance: linked,
                });
            }
        }
    }
    list.sort_by(|a, b| a.filename.cmp(&b.filename));
    list
}

#[tauri::command]
fn link_shared_item_to_instance(
    folder_name: String,
    instance_id: String,
    category: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cache_file = SharedStorageService::get_category_cache_dir(&category).join(&filename);
    if !cache_file.is_file() {
        return Err(format!("El archivo no existe en la caché compartida: {}", filename));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let instance_file = PathBuf::from(&home)
        .join(".omnicraft_studio")
        .join("instances")
        .join(&folder_name)
        .join(".minecraft")
        .join(&category)
        .join(&filename);

    SharedStorageService::link_file_to_instance(&cache_file, &instance_file)?;

    // Si es un mod, registrar en la base de datos de la instancia
    if category == "mods" && filename.ends_with(".jar") {
        let size = std::fs::metadata(&cache_file).map(|m| m.len()).unwrap_or(0);
        let mod_name = filename.replace(".jar", "");
        state.db.register_mod(
            &instance_id,
            &filename,
            None,
            &mod_name,
            None,
            None,
            size,
        ).ok();
    }

    Ok(())
}

#[tauri::command]
fn status() -> String {
    get_core_status().to_string()
}

#[tauri::command]
fn create_instance_cmd(
    name: String,
    mc_version: String,
    loader_type: String,
    icon: String,
    executable_type: String,
    launcher_jar: Option<String>,
    custom_exec: Option<String>,
    java_mode: String,
    custom_java: Option<String>,
    ram_min: u32,
    ram_max: u32,
    jvm_preset: Option<String>,
    jvm_args: Option<String>,
    instance_type: String,
    server_port: u16,
    assigned_username: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    state.db.create_instance(
        &name,
        &mc_version,
        &loader_type,
        &icon,
        &executable_type,
        launcher_jar.as_deref(),
        custom_exec.as_deref(),
        &java_mode,
        custom_java.as_deref(),
        ram_min,
        ram_max,
        jvm_preset.as_deref(),
        jvm_args.as_deref(),
        &instance_type,
        server_port,
        assigned_username.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_instance_cmd(
    id: String,
    name: String,
    icon: String,
    mc_version: String,
    loader_type: String,
    executable_type: String,
    launcher_jar: Option<String>,
    custom_exec: Option<String>,
    java_mode: String,
    custom_java: Option<String>,
    ram_min: u32,
    ram_max: u32,
    jvm_preset: Option<String>,
    jvm_args: Option<String>,
    width: u32,
    height: u32,
    assigned_username: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    state.db.update_instance(
        &id,
        &name,
        &icon,
        &mc_version,
        &loader_type,
        &executable_type,
        launcher_jar.as_deref(),
        custom_exec.as_deref(),
        &java_mode,
        custom_java.as_deref(),
        ram_min,
        ram_max,
        jvm_preset.as_deref(),
        jvm_args.as_deref(),
        width,
        height,
        assigned_username.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_instance_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    state.db.delete_instance(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_account_cmd(
    username: String,
    account_type: String,
    uuid: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    state.db.set_active_account(&username, &account_type, uuid.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_account_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    state.db.delete_account(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn logout_user_cmd(username: String, state: State<AppState>) -> Result<(), String> {
    state.db.logout_and_clear_user_data(&username).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_folder_cmd(folder_name: String, subfolder: Option<String>) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".omnicraft_studio");
    path.push("instances");
    path.push(&folder_name);
    path.push(".minecraft");
    if let Some(sub) = subfolder {
        path.push(sub);
    }
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn list_available_icons() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut p = PathBuf::from(&home);
    p.push("Games");
    p.push("Omnicraft-v2");
    p.push("icons");

    let mut res = Vec::new();
    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".png") && !name.contains("heberon") {
                res.push(name);
            }
        }
    }
    if res.is_empty() {
        let mut p_alt = PathBuf::from(&home);
        p_alt.push("Proyectos");
        p_alt.push("Omnicraft-v2");
        p_alt.push("public");
        p_alt.push("minecraft-icons");
        if let Ok(entries) = std::fs::read_dir(p_alt) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".png") && !name.contains("heberon") {
                    res.push(name);
                }
            }
        }
    }
    res.sort();
    res
}

#[tauri::command]
fn list_launchers() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut p = PathBuf::from(&home);
    p.push("Games");
    p.push("Omnicraft-v2");
    p.push("lanzadores");

    let mut res = Vec::new();
    // Primera opción recomendada: Motor Nativo Directo
    res.push("Motor Nativo Directo (Recomendado - Sin launchers externos)".to_string());

    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") {
                res.push(name);
            }
        }
    }
    res
}

#[tauri::command]
fn list_instance_worlds(folder_name: String) -> Result<Vec<WorldSaveInfo>, String> {
    ContentToolsService::list_worlds(&folder_name)
}

#[tauri::command]
fn backup_instance_world(folder_name: String, world_name: String) -> Result<String, String> {
    ContentToolsService::backup_world(&folder_name, &world_name)
}

#[tauri::command]
fn get_instance_logs(folder_name: String, max_lines: Option<usize>) -> Result<LogOutput, String> {
    ContentToolsService::read_instance_logs(&folder_name, max_lines.unwrap_or(500))
}

#[tauri::command]
fn export_modpack(folder_name: String, export_name: String, include_worlds: bool) -> Result<String, String> {
    ContentToolsService::export_instance_pack(&folder_name, &export_name, include_worlds)
}

#[tauri::command]
fn list_instance_content(folder_name: String, category: String) -> Result<Vec<ContentItem>, String> {
    ContentManagerService::list_category_items(&folder_name, &category)
}

#[tauri::command]
fn toggle_instance_content(
    folder_name: String,
    category: String,
    filename: String,
    enable: bool,
    instance_id: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let new_name = ContentManagerService::toggle_item_state(&folder_name, &category, &filename, enable)?;
    // Si es un mod y se proveyó instance_id, actualizar la tabla SQLite de mods
    if category == "mods" {
        if let Some(inst_id) = instance_id {
            let active_int = if enable { 1 } else { 0 };
            let _ = state.db.get_connection().map(|conn| {
                conn.execute(
                    "UPDATE instance_mods SET active = ?1, filename = ?2 WHERE instance_id = ?3 AND (filename = ?4 OR filename = ?2)",
                    rusqlite::params![active_int, new_name, inst_id, filename],
                ).ok();
            });
        }
    }
    Ok(new_name)
}

#[tauri::command]
fn delete_instance_content(
    folder_name: String,
    category: String,
    filename: String,
    instance_id: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    ContentManagerService::delete_item(&folder_name, &category, &filename)?;
    if category == "mods" {
        if let Some(inst_id) = instance_id {
            let _ = state.db.get_connection().map(|conn| {
                conn.execute(
                    "DELETE FROM instance_mods WHERE instance_id = ?1 AND (filename = ?2 OR filename = ?3)",
                    rusqlite::params![inst_id, filename, format!("{}.disabled", filename)],
                ).ok();
            });
        }
    }
    Ok(())
}

#[tauri::command]
fn restore_instance_world(folder_name: String, backup_filename: String) -> Result<String, String> {
    ContentToolsService::restore_world_backup(&folder_name, &backup_filename)
}

#[tauri::command]
fn scan_mod_file(folder_name: String, filename: String) -> Result<ScanResult, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut p = PathBuf::from(home);
    p.push(".omnicraft_studio");
    p.push("instances");
    p.push(folder_name);
    p.push(".minecraft");
    p.push("mods");
    p.push(filename);

    MalwareScannerService::scan_jar(&p)
}

#[tauri::command]
fn quarantine_mod_file(folder_name: String, filename: String) -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut p = PathBuf::from(home);
    p.push(".omnicraft_studio");
    p.push("instances");
    p.push(folder_name);
    p.push(".minecraft");
    p.push("mods");
    p.push(filename);

    MalwareScannerService::quarantine_mod(&p)
}

#[tauri::command]
async fn check_omnicraft_updates() -> Result<AppUpdateInfo, String> {
    UpdateCheckerService::check_for_updates().await
}

#[tauri::command]
async fn download_omnicraft_update_cmd(download_url: String, filename: String) -> Result<String, String> {
    UpdateCheckerService::download_update_installer(&download_url, &filename).await
}

#[tauri::command]
fn get_system_ui_mode(state: State<AppState>) -> String {
    state.db.get_setting("ui_mode").unwrap_or(None).unwrap_or_else(|| "basic".to_string())
}

#[tauri::command]
fn set_system_ui_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    state.db.set_setting("ui_mode", &mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_system_language(state: State<AppState>) -> String {
    state.db.get_setting("app_lang").unwrap_or(None).unwrap_or_else(|| "es".to_string())
}

#[tauri::command]
fn set_system_language(lang: String, state: State<AppState>) -> Result<(), String> {
    state.db.set_setting("app_lang", &lang).map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_fabric_loader_cmd(folder_name: String, mc_version: String) -> Result<String, String> {
    LoaderInstallerService::install_fabric_loader(&folder_name, &mc_version).await
}

#[tauri::command]
async fn download_paper_server_cmd(folder_name: String, mc_version: String) -> Result<String, String> {
    LoaderInstallerService::download_paper_server(&folder_name, &mc_version).await
}

#[tauri::command]
async fn install_temurin_jre_cmd(major_version: u32) -> Result<String, String> {
    JreProvisionerService::install_temurin_jre(major_version).await
}

#[tauri::command]
async fn start_ms_auth_flow() -> Result<MicrosoftDeviceCodeResponse, String> {
    MicrosoftAuthService::start_device_code_flow().await
}

#[tauri::command]
async fn poll_ms_auth_token(device_code: String, state: State<'_, AppState>) -> Result<MicrosoftAccountSession, String> {
    let session = MicrosoftAuthService::poll_device_token(&device_code).await?;
    state.db.save_microsoft_account(
        &session.username,
        &session.uuid,
        &session.access_token,
        session.refresh_token.as_deref(),
        session.skin_url.as_deref(),
    )
        .map_err(|e| format!("Error guardando sesión de Microsoft en BD: {}", e))?;
    Ok(session)
}

#[tauri::command]
fn assign_instance_account_cmd(
    instance_id: String,
    assigned_username: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let conn = state.db.get_connection().map_err(|e| e.to_string())?;
    let clean_user = assigned_username.and_then(|u| {
        let t = u.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    conn.execute(
        "UPDATE instances SET assigned_username = ?1 WHERE id = ?2",
        rusqlite::params![clean_user, instance_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    let app_state = AppState {
        db: DbManager::new(),
        engine: LaunchEngine::new(),
    };

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            let show_i = MenuItem::with_id(app, "show", "Abrir Omnicraft", true, None::<&str>)?;
            let restart_i = MenuItem::with_id(app, "restart", "Reiniciar Aplicación", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Cerrar del Todo", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[
                &show_i,
                &restart_i,
                &PredefinedMenuItem::separator(app)?,
                &quit_i,
            ])?;

            let tray_icon = app.default_window_icon().cloned();
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Omnicraft Launcher Studio v2.0")
                .show_menu_on_left_click(false);

            if let Some(icon) = tray_icon {
                tray_builder = tray_builder.icon(icon);
            }

            let _tray = tray_builder
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "restart" => {
                            app.restart();
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                        | TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // DevTools disponible mediante F12 / clic derecho o en depuración
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status,
            get_instances,
            get_accounts,
            get_official_recs,
            get_instance_mods,
            detect_javas,
            get_running_instances,
            get_instance_resource_usage,
            get_all_instances_resource_usage,
            terminate_instance,
            search_modrinth,
            parse_crash_log,
            setup_server_instance,
            sync_instance_profile,
            create_instance_cmd,
            update_instance_cmd,
            delete_instance_cmd,
            set_account_cmd,
            delete_account_cmd,
            logout_user_cmd,
            open_folder_cmd,
            open_url,
            list_available_icons,
            list_launchers,
            launch_instance_cmd,
            install_mod_to_instance,
            search_store_unified,
            get_api_key,
            set_api_key,
            get_all_game_versions,
            refresh_game_versions,
            get_local_skins,
            get_profile_skin_cmd,
            save_local_skin,
            delete_local_skin,
            assign_skin_to_profile_cmd,
            inject_offline_skin_to_instance,
            upload_skin_to_mojang,
            fetch_mojang_skin_cmd,
            search_remote_skins_cmd,
            download_remote_skin_cmd,
            list_instance_worlds,
            backup_instance_world,
            get_instance_logs,
            export_modpack,
            list_instance_content,
            toggle_instance_content,
            delete_instance_content,
            restore_instance_world,
            scan_mod_file,
            quarantine_mod_file,
            check_omnicraft_updates,
            download_omnicraft_update_cmd,
            get_system_ui_mode,
            set_system_ui_mode,
            get_system_language,
            set_system_language,
            install_fabric_loader_cmd,
            download_paper_server_cmd,
            send_server_cmd,
            install_temurin_jre_cmd,
            start_ms_auth_flow,
            poll_ms_auth_token,
            assign_instance_account_cmd,
            get_default_instance_id,
            set_default_instance_id,
            check_instance_compatibility,
            list_shared_cache_items,
            link_shared_item_to_instance
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Interceptar el cierre: ocultar a la bandeja en vez de destruir el proceso
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
