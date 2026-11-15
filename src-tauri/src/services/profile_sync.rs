// Inyector y sincronizador exhaustivo de perfiles de Minecraft y lanzadores en Rust
// Sincroniza launcher_profiles.json estándar, HMCL (formato Glavo y local/portable),
// SKlauncher, LauncherFenix y perfiles directos con herencia completa de JVM y usuario.
use std::path::{Path, PathBuf};
use std::fs;
use serde_json::{json, Value};
use crate::services::jvm_optimizer::JvmOptimizer;
use crate::services::java_service::JavaService;

pub struct ProfileSyncService;

impl ProfileSyncService {
    pub fn sync_instance_profile(
        instance_name: &str,
        mc_dir: &Path,
        mc_version: &str,
        loader_type: &str,
        java_bin: &str,
        ram_min_mb: u32,
        ram_max_mb: u32,
        jvm_preset: Option<&str>,
        jvm_args: &str,
        width: u32,
        height: u32,
        username: &str,
        uuid: Option<&str>,
        access_token: Option<&str>,
        skin_url: Option<&str>,
    ) -> Result<(), String> {
        fs::create_dir_all(mc_dir).map_err(|e| e.to_string())?;

        // Directorio raíz de la instancia (padre de .minecraft)
        let root_dir = mc_dir.parent().unwrap_or(mc_dir);

        // 0. Sincronizar SkinShuffle si el mod o su configuración existen en la instancia
        Self::sync_skinshuffle_config(mc_dir, username, uuid, skin_url);

        // 1. Resolver target version id exacto (si está fabric-loader-... o forge-... en versions/)
        let target_version_id = Self::resolve_version_id(mc_dir, mc_version, loader_type);

        // 2. Detectar versión mayor de Java a partir del binario o inferir por MC version
        let java_major = JavaService::inspect_java(Path::new(java_bin))
            .map(|j| j.version)
            .unwrap_or_else(|| JvmOptimizer::target_java_major(mc_version));

        // 3. Construir argumentos JVM inteligentemente ajustados según la versión
        let full_jvm_args = JvmOptimizer::build_jvm_args_string(
            mc_version,
            java_major,
            ram_min_mb,
            ram_max_mb,
            jvm_preset,
            jvm_args,
        );

        // =========================================================================
        // A. SINCRONIZAR launcher_profiles.json (Estándar Oficial, SKlauncher, Fenix)
        // Guarda versión de Minecraft/modloader, ruta java, jvm args y resolución
        // =========================================================================
        let profile_key = instance_name.replace(' ', "_");
        let now = chrono::Utc::now().to_rfc3339();

        let profile_obj = json!({
            "name": format!("Omnicraft - {}", instance_name),
            "type": "custom",
            "created": now,
            "lastUsed": now,
            "lastVersionId": target_version_id,
            "gameDir": mc_dir.to_string_lossy().to_string(),
            "javaDir": java_bin,
            "javaArgs": full_jvm_args,
            "resolution": {
                "width": if width > 0 { width } else { 1280 },
                "height": if height > 0 { height } else { 720 }
            }
        });

        for target_dir in &[mc_dir.to_path_buf(), root_dir.to_path_buf()] {
            let profiles_path = target_dir.join("launcher_profiles.json");
            let mut data: Value = if profiles_path.exists() {
                fs::read_to_string(&profiles_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| json!({"profiles": {}}))
            } else {
                json!({"profiles": {}})
            };

            if let Some(map) = data.get_mut("profiles").and_then(|p| p.as_object_mut()) {
                map.insert(profile_key.clone(), profile_obj.clone());
            }
            data["selectedProfile"] = json!(profile_key);

            if let Some(u) = uuid {
                let clean_uuid = u.replace('-', "");
                let token_val = access_token.unwrap_or("0");
                let mut auth_db = data.get("authenticationDatabase")
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();

                auth_db.insert(clean_uuid.clone(), json!({
                    "accessToken": token_val,
                    "username": username,
                    "profiles": {
                        clean_uuid.clone(): {
                            "displayName": username
                        }
                    }
                }));
                data["authenticationDatabase"] = json!(auth_db);
                data["selectedUser"] = json!({
                    "account": clean_uuid,
                    "profile": clean_uuid
                });
            }

            if let Ok(formatted) = serde_json::to_string_pretty(&data) {
                fs::write(&profiles_path, formatted).ok();
            }
        }

        // =========================================================================
        // B. SINCRONIZAR HMCL (Solo configuración de Java, Memoria, Directorios y Versiones)
        // Tanto en root_dir (donde arranca el proceso con HOME=root_dir) como en .minecraft
        // =========================================================================
        let hmcl_preset_id = format!("game-settings-preset:{}", instance_name.replace(' ', "_"));
        let hmcl_dir_id = format!("game-directory:{}", instance_name.replace(' ', "_"));

        // Configuración de juego de HMCL (Java bin, RAM, JVM args, resolución)
        let hmcl_game_settings_content = json!({
            "$schema": "https://schemas.glavo.site/hmcl/game-settings/1.0.0",
            "presets": [
                {
                    "id": hmcl_preset_id,
                    "name": format!("Omnicraft - {}", instance_name),
                    "autoMemory": false,
                    "minMemory": ram_min_mb,
                    "maxMemory": ram_max_mb,
                    "customJavaPath": java_bin,
                    "jvmOptions": full_jvm_args,
                    "width": width,
                    "height": height
                }
            ]
        });

        // Directorios de juego de HMCL
        let hmcl_game_dirs_content = json!({
            "$schema": "https://schemas.glavo.site/hmcl/game-directories/1.0.0",
            "directories": [
                {
                    "id": hmcl_dir_id,
                    "path": mc_dir.to_string_lossy().to_string()
                }
            ]
        });

        // Ajustes generales de HMCL seleccionando directorio y preset por defecto
        let hmcl_launcher_settings_content = json!({
            "$schema": "https://schemas.glavo.site/hmcl/launcher-settings/1.0.0",
            "selectedGameDirectory": hmcl_dir_id,
            "defaultGameSettingsPreset": hmcl_preset_id,
            "enableOfflineAccount": true,
            "theme": "DARK",
            "language": "es"
        });

        // Escribir configuración técnica de HMCL en todos los posibles árboles
        let hmcl_targets = vec![
            root_dir.join(".hmcl"),
            root_dir.join(".local").join("share").join("hmcl"),
            mc_dir.join(".hmcl"),
        ];

        for base in hmcl_targets {
            let config_dir = base.join("config");
            fs::create_dir_all(&config_dir).ok();

            fs::write(config_dir.join("game-settings.json"), serde_json::to_string_pretty(&hmcl_game_settings_content).unwrap_or_default()).ok();
            fs::write(config_dir.join("game-directories.json"), serde_json::to_string_pretty(&hmcl_game_dirs_content).unwrap_or_default()).ok();
            fs::write(config_dir.join("user-game-directories.json"), serde_json::to_string_pretty(&hmcl_game_dirs_content).unwrap_or_default()).ok();
            fs::write(config_dir.join("launcher-settings.json"), serde_json::to_string_pretty(&hmcl_launcher_settings_content).unwrap_or_default()).ok();
        }

        // =========================================================================
        // C. SINCRONIZAR LauncherFenix
        // Configura ruta de Java, RAM y directorio de juego
        // =========================================================================
        Self::sync_fenix_config(root_dir, mc_dir, ram_max_mb, java_bin);

        Ok(())
    }

    /// Sincroniza la configuración de SkinShuffle (si está instalado en la instancia)
    /// para que herede la skin oficial y el nombre de usuario de la sesión activa en vez de forzar a Steve ("Player").
    fn sync_skinshuffle_config(
        mc_dir: &Path,
        username: &str,
        uuid: Option<&str>,
        skin_url: Option<&str>,
    ) {
        let ss_dir = mc_dir.join("config").join("skinshuffle");
        let mods_dir = mc_dir.join("mods");
        let has_skinshuffle_mod = fs::read_dir(&mods_dir)
            .ok()
            .map(|entries| {
                entries.flatten().any(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    name.contains("skinshuffle") && name.ends_with(".jar")
                })
            })
            .unwrap_or(false);

        if !ss_dir.exists() && !has_skinshuffle_mod {
            return;
        }

        fs::create_dir_all(&ss_dir).ok();

        // 1. Asegurar que config.json tenga enableMultiAccountSupport: true y disableAPIUpload: true
        // (evita que MineSkin falle con HTTP 504 y sobreescriba la skin de Mojang con Steve)
        let cfg_path = ss_dir.join("config.json");
        let mut cfg_val: Value = if cfg_path.exists() {
            fs::read_to_string(&cfg_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!({
                "displayInPauseMenu": true,
                "displayInTitleScreen": true,
                "carouselScrollSensitivity": 1.0,
                "invertCarouselScroll": false,
                "widgetSkinRenderStyle": "CURSOR",
                "carouselSkinRenderStyle": "ROTATION",
                "presetEditScreenRenderStyle": "ROTATION",
                "rotationMultiplier": 1.0,
                "showCapeInPreview": true,
                "carouselView": "COMPACT",
                "welcomeGuideShown": true
            })
        };

        cfg_val["enableMultiAccountSupport"] = json!(true);
        cfg_val["disableAPIUpload"] = json!(true);
        cfg_val["welcomeGuideShown"] = json!(true);
        if let Ok(pretty) = serde_json::to_string_pretty(&cfg_val) {
            fs::write(&cfg_path, pretty).ok();
        }

        // 2. Determinar el preset de skin adecuado para el jugador activo
        // Si el usuario inyectó una skin local en config/skinshuffle/skins/<username>.png, preservarla
        let custom_skin_file = ss_dir.join("skins").join(format!("{}.png", username));
        let clean_uuid = uuid.unwrap_or("").replace('-', "");
        let formatted_uuid = if clean_uuid.len() == 32 {
            Some(format!(
                "{}-{}-{}-{}-{}",
                &clean_uuid[0..8],
                &clean_uuid[8..12],
                &clean_uuid[12..16],
                &clean_uuid[16..20],
                &clean_uuid[20..32]
            ))
        } else {
            None
        };

        let skin_entry = if let Some(url) = skin_url.filter(|u| !u.trim().is_empty()) {
            json!({
                "url": url,
                "model": "default",
                "type": "skinshuffle:url"
            })
        } else if custom_skin_file.exists() {
            json!({
                "skin_name": username,
                "model": "default",
                "type": "skinshuffle:config"
            })
        } else if let Some(u_dashed) = &formatted_uuid {
            json!({
                "uuid": u_dashed,
                "model": "default",
                "type": "skinshuffle:uuid"
            })
        } else {
            json!({
                "username": username,
                "model": "default",
                "type": "skinshuffle:username"
            })
        };

        let preset_payload = json!({
            "chosenPreset": 0,
            "apiPreset": 0,
            "loadedPresets": [
                {
                    "skin": skin_entry,
                    "name": username
                }
            ]
        });

        if let Ok(pretty_presets) = serde_json::to_string_pretty(&preset_payload) {
            // Actualizar presets.json global y presets-<username>.json específico de la cuenta
            let global_presets_path = ss_dir.join("presets.json");
            let user_presets_path = ss_dir.join(format!("presets-{}.json", username));

            // Si presets.json tenía el preset genérico "Player" o no existe, actualizarlo
            let should_overwrite_global = if global_presets_path.exists() {
                fs::read_to_string(&global_presets_path)
                    .map(|s| s.contains("\"name\": \"Player\"") || s.contains("1078232318") || s.contains(&format!("\"name\": \"{}\"", username)))
                    .unwrap_or(true)
            } else {
                true
            };

            if should_overwrite_global {
                fs::write(&global_presets_path, &pretty_presets).ok();
            }
            fs::write(&user_presets_path, &pretty_presets).ok();
        }

        // 3. Limpiar caché corrupta de Steve (31f477eb1a7beee631c2ca64d06f8f68fa93a3386d04452ab27f43acdf1b60cb)
        let caches_path = ss_dir.join("skin-caches.json");
        if caches_path.exists() {
            if let Ok(raw) = fs::read_to_string(&caches_path) {
                if raw.contains("31f477eb1a7beee631c2ca64d06f8f68fa93a3386d04452ab27f43acdf1b60cb") {
                    let clean_cache = json!({
                        "uploaded": {},
                        "downloaded": {}
                    });
                    if let Ok(p) = serde_json::to_string_pretty(&clean_cache) {
                        fs::write(&caches_path, p).ok();
                    }
                }
            }
        }
    }

    /// Sincroniza configuraciones de ejecución de LauncherFenix
    fn sync_fenix_config(root_dir: &Path, mc_dir: &Path, ram_mb: u32, java_bin: &str) {
        let fenix_dir = root_dir.join(".fenix");
        fs::create_dir_all(&fenix_dir).ok();

        let fenix_cfg = json!({
            "ram": ram_mb,
            "javaPath": java_bin,
            "gameDir": mc_dir.to_string_lossy().to_string()
        });

        fs::write(fenix_dir.join("config.json"), serde_json::to_string_pretty(&fenix_cfg).unwrap_or_default()).ok();
    }

    fn resolve_version_id(mc_dir: &Path, mc_version: &str, loader_type: &str) -> String {
        let versions_dir = mc_dir.join("versions");
        if !versions_dir.is_dir() {
            return mc_version.to_string();
        }

        let ldr_clean = loader_type.to_lowercase();
        if let Ok(entries) = fs::read_dir(&versions_dir) {
            let mut list = Vec::new();
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        list.push(name);
                    }
                }
            }

            if !ldr_clean.is_empty() && ldr_clean != "vanilla" {
                for v in &list {
                    let v_lower = v.to_lowercase();
                    if v_lower.contains(&ldr_clean) && v.contains(mc_version) {
                        return v.clone();
                    }
                }
                for v in &list {
                    if v.to_lowercase().contains(&ldr_clean) {
                        return v.clone();
                    }
                }
            }

            if list.contains(&mc_version.to_string()) {
                return mc_version.to_string();
            }

            if let Some(first) = list.first() {
                return first.clone();
            }
        }

        mc_version.to_string()
    }
}
