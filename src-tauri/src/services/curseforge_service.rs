// Cliente y motor de descargas para CurseForge API (Eternal API)
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_CURSEFORGE_KEY: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";
pub const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    pub id: String,
    pub source: String, // "Modrinth" | "CurseForge"
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub web_url: String,
    pub loaders: Vec<String>,
    pub versions: Vec<String>,
}

pub struct CurseForgeService;

impl CurseForgeService {
    pub async fn search(
        query: &str,
        project_type: &str,
        mc_version: &str,
        loader: &str,
        limit: u32,
        api_key: Option<&str>,
    ) -> Result<Vec<UnifiedSearchResult>, String> {
        let key = api_key.unwrap_or(DEFAULT_CURSEFORGE_KEY);

        // Mapeo classId: 6 = Mods, 4471 = Modpacks, 12 = Resource Packs, 6552 = Shaders
        let class_id = match project_type {
            "shader" => 6552,
            "resourcepack" => 12,
            "modpack" => 4471,
            _ => 6,
        };

        // Mapeo loader: 1 = Forge, 4 = Fabric, 5 = Quilt, 6 = NeoForge
        let loader_id: Option<u32> = match loader.to_lowercase().as_str() {
            "forge" => Some(1),
            "fabric" => Some(4),
            "quilt" => Some(5),
            "neoforge" => Some(6),
            _ => None,
        };

        let mut url = format!(
            "{}/mods/search?gameId=432&classId={}&searchFilter={}&pageSize={}&sortField=5&sortOrder=desc",
            CURSEFORGE_API,
            class_id,
            urlencoding::encode(query),
            limit
        );

        if !mc_version.is_empty() && mc_version != "Todas" {
            url.push_str(&format!("&gameVersion={}", urlencoding::encode(mc_version)));
        }

        if let Some(lid) = loader_id {
            url.push_str(&format!("&modLoaderType={}", lid));
        }

        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let res = client
            .get(&url)
            .header("x-api-key", key)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("CurseForge retornó status HTTP {}", res.status()));
        }

        let val: Value = res.json().await.map_err(|e| e.to_string())?;
        let data = val.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let mut results = Vec::new();
        for m in data {
            let id = m.get("id").and_then(|v| v.as_i64()).map(|n| n.to_string()).unwrap_or_default();
            let slug = m.get("slug").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
            let title = m.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let description = m.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();

            let authors_str = m.get("authors")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|auth| auth.get("name").and_then(|n| n.as_str()))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "Autor".to_string());

            let downloads = m.get("downloadCount").and_then(|v| v.as_u64()).unwrap_or(0);
            let icon_url = m.get("logo")
                .and_then(|l| l.get("thumbnailUrl"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());

            let web_url = format!("https://www.curseforge.com/minecraft/mc-mods/{}", slug);

            let mut loaders = Vec::new();
            if !loader.is_empty() && loader != "Todos" {
                loaders.push(loader.to_string());
            }

            results.push(UnifiedSearchResult {
                id,
                source: "CurseForge".to_string(),
                slug,
                title,
                description,
                author: authors_str,
                downloads,
                icon_url,
                project_type: project_type.to_string(),
                web_url,
                loaders,
                versions: Vec::new(),
            });
        }

        Ok(results)
    }

    pub async fn download_mod(
        mod_id: &str,
        mc_version: &str,
        loader: &str,
        dest_folder: &Path,
        api_key: Option<&str>,
    ) -> Result<String, String> {
        Self::download_item(mod_id, mc_version, loader, dest_folder, "mods", api_key).await
    }

    pub async fn download_item(
        mod_id: &str,
        mc_version: &str,
        loader: &str,
        dest_folder: &Path,
        category: &str,
        api_key: Option<&str>,
    ) -> Result<String, String> {
        let key = api_key.unwrap_or(DEFAULT_CURSEFORGE_KEY);
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let loader_id: Option<u32> = match loader.to_lowercase().as_str() {
            "forge" => Some(1),
            "fabric" => Some(4),
            "quilt" => Some(5),
            "neoforge" => Some(6),
            _ => None,
        };

        // Si es modpack, no filtrar rígidamente por loaderId en CurseForge ya que a veces lo etiquetan de forma mixta
        let mut url = format!("{}/mods/{}/files?pageSize=25", CURSEFORGE_API, mod_id);
        if !mc_version.is_empty() && mc_version != "Todas" {
            url.push_str(&format!("&gameVersion={}", urlencoding::encode(mc_version)));
        }
        if category != "modpacks" {
            if let Some(lid) = loader_id {
                url.push_str(&format!("&modLoaderType={}", lid));
            }
        }

        let res = client
            .get(&url)
            .header("x-api-key", key)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("CurseForge files retornó status HTTP {}", res.status()));
        }

        let val: Value = res.json().await.map_err(|e| e.to_string())?;
        let mut files = val.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
        
        // Si no encontró con el filtro estricto de versión, reintentar sin filtrar gameVersion para no abortar
        if files.is_empty() {
            let fallback_url = format!("{}/mods/{}/files?pageSize=10", CURSEFORGE_API, mod_id);
            if let Ok(fallback_res) = client.get(&fallback_url).header("x-api-key", key).send().await {
                if let Ok(fallback_val) = fallback_res.json::<Value>().await {
                    files = fallback_val.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
                }
            }
        }

        if files.is_empty() {
            return Err("No hay archivos compatibles en CurseForge para este recurso".to_string());
        }

        let file = &files[0];
        let file_id = file.get("id").and_then(|v| v.as_i64()).ok_or("ID de archivo no válido")?;
        let filename = file.get("fileName").and_then(|v| v.as_str()).unwrap_or("resource.zip").to_string();

        let final_path = dest_folder.join(&filename);

        // Si ya existe en la caché compartida, enlazar con Hardlink (0 MB extra)
        let cache_file = crate::services::shared_storage::SharedStorageService::get_category_cache_dir(category).join(&filename);
        if cache_file.is_file() {
            crate::services::shared_storage::SharedStorageService::link_file_to_instance(&cache_file, &final_path)?;
            return Ok(filename);
        }

        // Obtener URL de descarga directa
        let mut download_url = file.get("downloadUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
        if download_url.is_none() {
            let dl_info_url = format!("{}/mods/{}/files/{}/download-url", CURSEFORGE_API, mod_id, file_id);
            let dl_resp = client
                .get(&dl_info_url)
                .header("x-api-key", key)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let dl_val: Value = dl_resp.json().await.map_err(|e| e.to_string())?;
            download_url = dl_val
                .get("data")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        let dl_url_final = download_url.ok_or("CurseForge no proveyó URL de descarga directa para este archivo")?;

        let file_bytes = client.get(&dl_url_final).send().await.map_err(|e| e.to_string())?.bytes().await.map_err(|e| e.to_string())?;

        crate::services::shared_storage::SharedStorageService::store_and_link(
            category,
            &filename,
            &file_bytes,
            &final_path,
        )?;

        // Resolución automática y recursiva de dependencias obligatorias en CurseForge (relationType == 3)
        if category == "mods" {
            if let Some(deps) = file.get("dependencies").and_then(|d| d.as_array()) {
                for dep in deps {
                    let rel_type = dep.get("relationType").and_then(|r| r.as_i64()).unwrap_or(0);
                    if rel_type == 3 {
                        if let Some(dep_mod_id) = dep.get("modId").and_then(|m| m.as_i64()) {
                            let dep_id_str = dep_mod_id.to_string();
                            Box::pin(Self::download_mod(
                                &dep_id_str,
                                mc_version,
                                loader,
                                dest_folder,
                                api_key,
                            )).await.ok();
                        }
                    }
                }
            }
        }

        Ok(filename)
    }
}
