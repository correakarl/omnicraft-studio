// Motor de Lanzamiento Directo Nativo de Minecraft en Rust para Omnicraft
// Resuelve versiones (Vanilla, Fabric, Forge, NeoForge, Quilt), construye el Classpath
// deduplicado y ejecuta el cliente de juego directamente pasando tokens libres o Microsoft.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

pub struct DirectLaunchConfig<'a> {
    pub instance_name: &'a str,
    pub mc_dir: &'a Path,
    pub mc_version: &'a str,
    pub loader_type: &'a str,
    pub java_bin: &'a str,
    pub jvm_args: &'a [String],
    pub username: &'a str,
    pub uuid: Option<&'a str>,
    pub access_token: Option<&'a str>,
    pub user_type: &'a str, // "msa" o "legacy"
    pub width: u32,
    pub height: u32,
}

pub struct DirectLaunchPlan {
    pub main_class: String,
    pub classpath: String,
    pub game_args: Vec<String>,
}

impl DirectLaunchPlan {
    /// Limpia archivos .json dentro de config/ que tengan comas finales (trailing commas) o entradas null/vacías en listas
    fn sanitize_instance_configs(mc_dir: &Path) {
        let config_dir = mc_dir.join("config");
        if !config_dir.is_dir() {
            return;
        }
        if let Ok(entries) = fs::read_dir(&config_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&p) {
                        let mut cleaned = String::with_capacity(content.len());
                        let chars: Vec<char> = content.chars().collect();
                        let mut i = 0;
                        while i < chars.len() {
                            if chars[i] == ',' {
                                let mut j = i + 1;
                                while j < chars.len() && chars[j].is_whitespace() {
                                    j += 1;
                                }
                                if j < chars.len() && (chars[j] == ']' || chars[j] == '}') {
                                    i += 1;
                                    continue;
                                }
                            }
                            cleaned.push(chars[i]);
                            i += 1;
                        }

                        if let Ok(mut val) = serde_json::from_str::<Value>(&cleaned) {
                            let mut modified = cleaned != content;
                            if let Some(obj) = val.as_object_mut() {
                                for (_k, v) in obj.iter_mut() {
                                    if let Some(arr) = v.as_array_mut() {
                                        let orig_len = arr.len();
                                        arr.retain(|item| !item.is_null() && item.as_str() != Some(""));
                                        if arr.len() != orig_len {
                                            modified = true;
                                        }
                                    }
                                }
                            }
                            if modified {
                                if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                                    let _ = fs::write(&p, pretty);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Resuelve el plan de ejecución analizando la versión y su herencia
    pub fn build(config: &DirectLaunchConfig) -> Result<Self, String> {
        Self::sanitize_instance_configs(config.mc_dir);

        let versions_dir = config.mc_dir.join("versions");
        if !versions_dir.exists() {
            return Err("No existe el directorio de versiones 'versions' en la instancia.".to_string());
        }

        // 1. Encontrar el archivo version.json target
        let target_ver_id = Self::find_target_version_id(&versions_dir, config.mc_version, config.loader_type)?;
        let target_json_path = versions_dir.join(&target_ver_id).join(format!("{}.json", target_ver_id));

        if !target_json_path.exists() {
            return Err(format!("No se encontró el descriptor de versión: {}", target_json_path.display()));
        }

        let target_raw = fs::read_to_string(&target_json_path)
            .map_err(|e| format!("Error leyendo {}: {}", target_json_path.display(), e))?;
        let target_data: Value = serde_json::from_str(&target_raw)
            .map_err(|e| format!("JSON corrupto en {}: {}", target_json_path.display(), e))?;

        // 2. Comprobar si hereda de una versión padre (ej. Fabric/Forge hereda de Vanilla)
        let parent_data: Option<Value> = if let Some(parent_id) = target_data.get("inheritsFrom").and_then(|v| v.as_str()) {
            let parent_json = versions_dir.join(parent_id).join(format!("{}.json", parent_id));
            if parent_json.exists() {
                fs::read_to_string(&parent_json)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            } else {
                None
            }
        } else {
            None
        };

        // 3. Determinar mainClass
        let main_class = target_data.get("mainClass")
            .and_then(|v| v.as_str())
            .or_else(|| parent_data.as_ref().and_then(|p| p.get("mainClass").and_then(|v| v.as_str())))
            .unwrap_or("net.minecraft.client.main.Main")
            .to_string();

        // 4. Resolver librerías con deduplicación y reglas OS/Arch
        let libraries_dir = config.mc_dir.join("libraries");
        let mut cp_map: HashMap<(String, String, Option<String>), PathBuf> = HashMap::new();

        // Procesar primero el padre (base) y luego la versión target (para que el modloader sobreescriba ASM/Guava etc)
        let mut library_lists = Vec::new();
        if let Some(p) = &parent_data {
            if let Some(libs) = p.get("libraries").and_then(|v| v.as_array()) {
                library_lists.push(libs);
            }
        }
        if let Some(libs) = target_data.get("libraries").and_then(|v| v.as_array()) {
            library_lists.push(libs);
        }

        for libs in library_lists {
            for lib in libs {
                if !Self::check_library_rules(lib.get("rules")) {
                    continue;
                }

                let name = match lib.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => continue,
                };

                let parts: Vec<&str> = name.split(':').collect();
                if parts.len() < 3 {
                    continue;
                }
                let group = parts[0];
                let artifact = parts[1];
                let version = parts[2];
                let classifier = if parts.len() > 3 { Some(parts[3]) } else { None };

                let rel_path = if let Some(p) = lib.get("downloads")
                    .and_then(|d| d.get("artifact"))
                    .and_then(|a| a.get("path"))
                    .and_then(|p| p.as_str()) {
                    p.to_string()
                } else {
                    let domain = group.replace('.', "/");
                    if let Some(c) = classifier {
                        format!("{}/{}/{}/{}-{}-{}.jar", domain, artifact, version, artifact, version, c)
                    } else {
                        format!("{}/{}/{}/{}-{}.jar", domain, artifact, version, artifact, version)
                    }
                };

                let full_path = libraries_dir.join(&rel_path);
                if full_path.exists() {
                    let key = (group.to_string(), artifact.to_string(), classifier.map(|s| s.to_string()));
                    cp_map.insert(key, full_path);
                }
            }
        }

        let mut cp_elements: Vec<PathBuf> = cp_map.into_values().collect();

        // 5. Agregar el JAR del cliente principal (ej. versions/1.21.4/1.21.4.jar)
        let client_ver_id = parent_data.as_ref()
            .and_then(|p| p.get("id").and_then(|v| v.as_str()))
            .unwrap_or(&target_ver_id);

        let client_jar = versions_dir.join(client_ver_id).join(format!("{}.jar", client_ver_id));
        if client_jar.exists() {
            cp_elements.push(client_jar);
        } else {
            // Si no está en el target ni padre, intentar el mc_version original
            let fallback_jar = versions_dir.join(config.mc_version).join(format!("{}.jar", config.mc_version));
            if fallback_jar.exists() {
                cp_elements.push(fallback_jar);
            }
        }

        let sep = if cfg!(windows) { ";" } else { ":" };
        let classpath = cp_elements
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(sep);

        // 6. Ensamblar argumentos del juego
        let asset_index = parent_data.as_ref()
            .and_then(|p| p.get("assetIndex").and_then(|a| a.get("id")).and_then(|v| v.as_str()))
            .or_else(|| target_data.get("assetIndex").and_then(|a| a.get("id")).and_then(|v| v.as_str()))
            .unwrap_or(config.mc_version);

        let assets_dir = config.mc_dir.join("assets");
        let default_uuid = format!("00000000-0000-0000-0000-{:012x}", (config.username.len() as u64));
        let player_uuid = config.uuid.unwrap_or(&default_uuid);
        let token = config.access_token.unwrap_or("0");

        let mut game_args = vec![
            "--username".to_string(),
            config.username.to_string(),
            "--version".to_string(),
            config.mc_version.to_string(),
            "--gameDir".to_string(),
            config.mc_dir.to_string_lossy().to_string(),
            "--assetsDir".to_string(),
            assets_dir.to_string_lossy().to_string(),
            "--assetIndex".to_string(),
            asset_index.to_string(),
            "--uuid".to_string(),
            player_uuid.to_string(),
            "--accessToken".to_string(),
            token.to_string(),
            "--userType".to_string(),
            config.user_type.to_string(),
            "--versionType".to_string(),
            "release".to_string(),
        ];

        let width = if config.width > 0 { config.width } else { 1280 };
        let height = if config.height > 0 { config.height } else { 720 };
        game_args.push("--width".to_string());
        game_args.push(width.to_string());
        game_args.push("--height".to_string());
        game_args.push(height.to_string());

        Ok(Self {
            main_class,
            classpath,
            game_args,
        })
    }

    /// Resuelve el ID de versión adecuada en la carpeta versions
    fn find_target_version_id(versions_dir: &Path, mc_version: &str, loader_type: &str) -> Result<String, String> {
        let loader_lower = loader_type.to_lowercase();
        let is_modded = loader_lower != "vanilla" && loader_lower != "none" && !loader_lower.is_empty();

        if let Ok(entries) = fs::read_dir(versions_dir) {
            let mut candidates: Vec<String> = Vec::new();
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        candidates.push(name);
                    }
                }
            }

            // Si es modded, buscar primero la que contenga el loader y la versión
            if is_modded {
                for c in &candidates {
                    let cl = c.to_lowercase();
                    if cl.contains(&loader_lower) && cl.contains(mc_version) {
                        return Ok(c.clone());
                    }
                }
                for c in &candidates {
                    let cl = c.to_lowercase();
                    if cl.contains(&loader_lower) {
                        return Ok(c.clone());
                    }
                }
            }

            // Buscar versión exacta
            for c in &candidates {
                if c == mc_version {
                    return Ok(c.clone());
                }
            }

            // Si hay alguna candidata disponible
            if let Some(first) = candidates.first() {
                return Ok(first.clone());
            }
        }

        Ok(mc_version.to_string())
    }

    /// Valida si una regla de biblioteca aplica a la plataforma actual
    fn check_library_rules(rules_val: Option<&Value>) -> bool {
        let rules = match rules_val.and_then(|v| v.as_array()) {
            Some(r) => r,
            None => return true,
        };

        let current_os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "unknown"
        };

        let current_arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "unknown"
        };

        let mut allowed = false;

        for r in rules {
            let action = r.get("action").and_then(|v| v.as_str()).unwrap_or("allow");
            let os_spec = r.get("os");

            let matches_os = if let Some(os) = os_spec {
                let name_match = os.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n == current_os)
                    .unwrap_or(true);

                let arch_match = os.get("arch")
                    .and_then(|a| a.as_str())
                    .map(|a| a == current_arch)
                    .unwrap_or(true);

                name_match && arch_match
            } else {
                true
            };

            if action == "allow" && matches_os {
                allowed = true;
            } else if action == "disallow" && matches_os {
                allowed = false;
            }
        }

        allowed
    }
}
