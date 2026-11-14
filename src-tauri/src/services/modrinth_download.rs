// Descargador y resolutor atómico de versiones para Modrinth
use std::path::Path;
use serde_json::Value;
use crate::services::shared_storage::SharedStorageService;

pub struct ModrinthDownloadService;

impl ModrinthDownloadService {
    pub async fn download_project(
        project_id_or_slug: &str,
        mc_version: &str,
        loader: &str,
        dest_folder: &Path,
        category: &str,
    ) -> Result<String, String> {
        let url = format!("https://api.modrinth.com/v2/project/{}/version", project_id_or_slug);
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Modrinth retornó código {}", res.status()));
        }

        let versions: Vec<Value> = res.json().await.map_err(|e| e.to_string())?;
        if versions.is_empty() {
            return Err(format!("No hay versiones disponibles para {}", project_id_or_slug));
        }

        // Buscar versión que coincida con mc_version y loader
        let mut target_ver = None;
        let ldr_clean = loader.to_lowercase();

        for v in &versions {
            let gv = v.get("game_versions").and_then(|a| a.as_array());
            let ld = v.get("loaders").and_then(|a| a.as_array());

            let mc_matches = if mc_version.is_empty() || mc_version == "Todas" {
                true
            } else {
                gv.map(|arr| arr.iter().any(|val| val.as_str() == Some(mc_version))).unwrap_or(false)
            };

            let loader_matches = if ldr_clean.is_empty() || ldr_clean == "todos" || ldr_clean == "vanilla" || category == "modpacks" {
                true
            } else {
                ld.map(|arr| arr.iter().any(|val| {
                    let s = val.as_str().map(|s| s.to_lowercase()).unwrap_or_default();
                    s == ldr_clean || (ldr_clean == "quilt" && s == "fabric") || (ldr_clean == "neoforge" && s == "forge")
                })).unwrap_or(false)
            };

            if mc_matches && loader_matches {
                target_ver = Some(v);
                break;
            }
        }

        // Si no hubo coincidencia exacta con versión y loader, intentar coincidir al menos la versión de Minecraft
        if target_ver.is_none() && !mc_version.is_empty() && mc_version != "Todas" {
            for v in &versions {
                let gv = v.get("game_versions").and_then(|a| a.as_array());
                if gv.map(|arr| arr.iter().any(|val| val.as_str() == Some(mc_version))).unwrap_or(false) {
                    target_ver = Some(v);
                    break;
                }
            }
        }

        let selected = match target_ver {
            Some(v) => v,
            None => {
                // Si el usuario especificó una versión o loader específico y no existe absolutamente ninguna versión compatible:
                if (!mc_version.is_empty() && mc_version != "Todas") || (!ldr_clean.is_empty() && ldr_clean != "todos" && ldr_clean != "vanilla") {
                    return Err(format!(
                        "Incompatibilidad en Modrinth: No se encontró versión para Minecraft {} con Loader {}.",
                        if mc_version.is_empty() { "cualquiera" } else { mc_version },
                        if ldr_clean.is_empty() { "cualquiera" } else { &ldr_clean }
                    ));
                }
                &versions[0]
            }
        };

        let files = selected.get("files").and_then(|f| f.as_array()).ok_or("No hay archivos en la versión")?;

        // En caso de modpacks, priorizar archivos .mrpack sobre bundles o changelogs
        let primary_file = if category == "modpacks" {
            files.iter()
                .find(|f| f.get("filename").and_then(|n| n.as_str()).map(|n| n.ends_with(".mrpack")).unwrap_or(false))
                .or_else(|| files.iter().find(|f| f.get("primary").and_then(|p| p.as_bool()).unwrap_or(false)))
                .unwrap_or(&files[0])
        } else {
            files.iter()
                .find(|f| f.get("primary").and_then(|p| p.as_bool()).unwrap_or(false))
                .unwrap_or(&files[0])
        };

        let dl_url = primary_file.get("url").and_then(|u| u.as_str()).ok_or("URL de descarga no encontrada")?;
        let filename = primary_file.get("filename").and_then(|n| n.as_str()).ok_or("Nombre de archivo no encontrado")?;

        let final_path = dest_folder.join(filename);

        // Si ya está en la caché compartida, simplemente vincular por Hardlink (0 MB extra)
        let cache_file = SharedStorageService::get_category_cache_dir(category).join(filename);
        if cache_file.is_file() {
            SharedStorageService::link_file_to_instance(&cache_file, &final_path)?;
        } else {
            // Descargar y guardar en caché compartida vinculando atómicamente a la instancia
            let file_resp = client.get(dl_url).send().await.map_err(|e| e.to_string())?;
            let bytes = file_resp.bytes().await.map_err(|e| e.to_string())?;
            SharedStorageService::store_and_link(category, filename, &bytes, &final_path)?;
        }

        // 3. Resolución automática de dependencias obligatorias (Plus Ultra Engine)
        if let Some(deps) = selected.get("dependencies").and_then(|d| d.as_array()) {
            for dep in deps {
                let dep_type = dep.get("dependency_type").and_then(|t| t.as_str()).unwrap_or("");
                if dep_type == "required" {
                    if let Some(dep_proj_id) = dep.get("project_id").and_then(|p| p.as_str()) {
                        // Descargar recursivamente la dependencia obligatoria
                        Box::pin(Self::download_project(dep_proj_id, mc_version, loader, dest_folder, category)).await.ok();
                    }
                }
            }
        }

        Ok(filename.to_string())
    }
}
