// Servicio Unificado de Extracción e Instalación de Modpacks (.mrpack [Modrinth] y .zip [CurseForge])
use std::path::Path;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

// --- Modelos Modrinth (.mrpack) ---
#[derive(Debug, Deserialize)]
pub struct MrpackIndex {
    pub name: String,
    pub files: Vec<MrpackFile>,
}

#[derive(Debug, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    pub downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
}

// --- Modelos CurseForge (.zip / manifest.json) ---
#[derive(Debug, Deserialize)]
pub struct CurseForgeManifest {
    pub name: String,
    pub version: Option<String>,
    #[serde(default)]
    pub files: Vec<CurseForgeFileItem>,
    pub overrides: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CurseForgeFileItem {
    #[serde(rename = "projectID")]
    pub project_id: i64,
    #[serde(rename = "fileID")]
    pub file_id: i64,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModpackInstallProgress {
    pub step: String,
    pub current_file: String,
    pub completed_files: usize,
    pub total_files: usize,
    pub percent: f32,
}

pub struct ModpackInstallerService;

impl ModpackInstallerService {
    /// Detecta e instala un modpack sea de Modrinth (.mrpack) o de CurseForge (.zip)
    pub async fn install_modpack_auto<F>(
        archive_path: &Path,
        minecraft_dir: &Path,
        cf_api_key: Option<&str>,
        progress_cb: F,
    ) -> Result<String, String>
    where
        F: FnMut(ModpackInstallProgress) + Send + 'static,
    {
        let file = File::open(archive_path)
            .map_err(|e| format!("Error abriendo paquete {}: {}", archive_path.display(), e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Formato de archivo zip/mrpack inválido: {}", e))?;

        let has_mrpack = archive.by_name("modrinth.index.json").is_ok();
        let has_manifest = archive.by_name("manifest.json").is_ok();

        drop(archive);

        if has_mrpack {
            Self::install_mrpack(archive_path, minecraft_dir, progress_cb).await
        } else if has_manifest {
            Self::install_curseforge_pack(archive_path, minecraft_dir, cf_api_key, progress_cb).await
        } else {
            Err("El archivo descargado no contiene un manifiesto válido (modrinth.index.json o manifest.json)".to_string())
        }
    }

    /// Descomprime overrides de un .mrpack y descarga los mods listados en modrinth.index.json
    pub async fn install_mrpack<F>(
        mrpack_path: &Path,
        minecraft_dir: &Path,
        mut progress_cb: F,
    ) -> Result<String, String>
    where
        F: FnMut(ModpackInstallProgress) + Send + 'static,
    {
        let file = File::open(mrpack_path)
            .map_err(|e| format!("Error abriendo {}: {}", mrpack_path.display(), e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Formato mrpack zip inválido: {}", e))?;

        // 1. Leer modrinth.index.json
        let mut index_file = archive.by_name("modrinth.index.json")
            .map_err(|_| "El archivo modpack no contiene modrinth.index.json válido".to_string())?;
        
        let mut index_content = String::new();
        index_file.read_to_string(&mut index_content).map_err(|e| e.to_string())?;
        let index: MrpackIndex = serde_json::from_str(&index_content)
            .map_err(|e| format!("Error parseando modrinth.index.json: {}", e))?;

        drop(index_file);

        let total_files = index.files.len() + 1;
        let mut completed = 0;

        progress_cb(ModpackInstallProgress {
            step: "Extrayendo configuraciones (overrides)...".to_string(),
            current_file: "overrides".to_string(),
            completed_files: completed,
            total_files,
            percent: 5.0,
        });

        // 2. Extraer overrides
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            if name.starts_with("overrides/") {
                let subpath = &name["overrides/".len()..];
                if !subpath.is_empty() {
                    let outpath = minecraft_dir.join(subpath);
                    if file.is_dir() {
                        std::fs::create_dir_all(&outpath).ok();
                    } else {
                        if let Some(p) = outpath.parent() {
                            std::fs::create_dir_all(p).ok();
                        }
                        let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        completed += 1;

        // 3. Descargar cada archivo declarado en index.files con cliente optimizado
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        for file_info in &index.files {
            if let Some(download_url) = file_info.downloads.first() {
                let dest_path = minecraft_dir.join(&file_info.path);
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                progress_cb(ModpackInstallProgress {
                    step: format!("Descargando mods ({}/{})", completed, total_files),
                    current_file: file_info.path.clone(),
                    completed_files: completed,
                    total_files,
                    percent: (completed as f32 / total_files as f32) * 100.0,
                });

                let filename = Path::new(&file_info.path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("mod.jar");

                let category = if file_info.path.contains("shader") {
                    "shaderpacks"
                } else if file_info.path.contains("resource") {
                    "resourcepacks"
                } else {
                    "mods"
                };

                let cached_file = crate::services::shared_storage::SharedStorageService::get_category_cache_dir(category).join(filename);

                if !dest_path.exists() {
                    if cached_file.is_file() {
                        crate::services::shared_storage::SharedStorageService::link_file_to_instance(&cached_file, &dest_path).ok();
                    } else if let Ok(resp) = client.get(download_url).send().await {
                        if let Ok(bytes) = resp.bytes().await {
                            crate::services::shared_storage::SharedStorageService::store_and_link(category, filename, &bytes, &dest_path).ok();
                        }
                    }
                }
            }
            completed += 1;
        }

        progress_cb(ModpackInstallProgress {
            step: "¡Modpack instalado correctamente!".to_string(),
            current_file: index.name.clone(),
            completed_files: total_files,
            total_files,
            percent: 100.0,
        });

        Ok(index.name)
    }

    /// Descomprime overrides de un paquete CurseForge (.zip) y descarga los mods listados en manifest.json
    pub async fn install_curseforge_pack<F>(
        zip_path: &Path,
        minecraft_dir: &Path,
        cf_api_key: Option<&str>,
        mut progress_cb: F,
    ) -> Result<String, String>
    where
        F: FnMut(ModpackInstallProgress) + Send + 'static,
    {
        use crate::services::curseforge_service::{CURSEFORGE_API, DEFAULT_CURSEFORGE_KEY};

        let file = File::open(zip_path)
            .map_err(|e| format!("Error abriendo {}: {}", zip_path.display(), e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Formato zip inválido: {}", e))?;

        // 1. Leer manifest.json
        let mut manifest_file = archive.by_name("manifest.json")
            .map_err(|_| "El modpack de CurseForge no contiene manifest.json".to_string())?;
        
        let mut manifest_content = String::new();
        manifest_file.read_to_string(&mut manifest_content).map_err(|e| e.to_string())?;
        let manifest: CurseForgeManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Error parseando manifest.json de CurseForge: {}", e))?;

        drop(manifest_file);

        let overrides_folder = manifest.overrides.clone().unwrap_or_else(|| "overrides".to_string());
        let overrides_prefix = format!("{}/", overrides_folder);

        let total_files = manifest.files.len() + 1;
        let mut completed = 0;

        progress_cb(ModpackInstallProgress {
            step: "Extrayendo configuraciones y recursos de CurseForge...".to_string(),
            current_file: "overrides".to_string(),
            completed_files: completed,
            total_files,
            percent: 5.0,
        });

        // 2. Extraer carpeta overrides
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            if name.starts_with(&overrides_prefix) {
                let subpath = &name[overrides_prefix.len()..];
                if !subpath.is_empty() {
                    let outpath = minecraft_dir.join(subpath);
                    if file.is_dir() {
                        std::fs::create_dir_all(&outpath).ok();
                    } else {
                        if let Some(p) = outpath.parent() {
                            std::fs::create_dir_all(p).ok();
                        }
                        let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        completed += 1;

        // 3. Descargar mods de CurseForge mediante su API oficial
        let key = cf_api_key.unwrap_or(DEFAULT_CURSEFORGE_KEY);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let mods_dir = minecraft_dir.join("mods");
        std::fs::create_dir_all(&mods_dir).ok();

        for item in &manifest.files {
            progress_cb(ModpackInstallProgress {
                step: format!("Descargando mods de CurseForge ({}/{})", completed, total_files),
                current_file: format!("Mod ID: {}", item.project_id),
                completed_files: completed,
                total_files,
                percent: (completed as f32 / total_files as f32) * 100.0,
            });

            // A) Obtener información del archivo para conocer su nombre exacto
            let file_info_url = format!("{}/mods/{}/files/{}", CURSEFORGE_API, item.project_id, item.file_id);
            if let Ok(res) = client.get(&file_info_url).header("x-api-key", key).send().await {
                if let Ok(val) = res.json::<serde_json::Value>().await {
                    let file_data = val.get("data").cloned().unwrap_or(val);
                    let filename = file_data.get("fileName").and_then(|f| f.as_str()).unwrap_or("mod.jar");
                    let dest_path = mods_dir.join(filename);

                    let cached_file = crate::services::shared_storage::SharedStorageService::get_category_cache_dir("mods").join(filename);

                    if !dest_path.exists() {
                        if cached_file.is_file() {
                            crate::services::shared_storage::SharedStorageService::link_file_to_instance(&cached_file, &dest_path).ok();
                        } else {
                            // B) Obtener URL de descarga directa
                            let mut download_url = file_data.get("downloadUrl").and_then(|d| d.as_str()).map(|s| s.to_string());
                            if download_url.is_none() {
                                let dl_api_url = format!("{}/mods/{}/files/{}/download-url", CURSEFORGE_API, item.project_id, item.file_id);
                                if let Ok(dl_res) = client.get(&dl_api_url).header("x-api-key", key).send().await {
                                    if let Ok(dl_val) = dl_res.json::<serde_json::Value>().await {
                                        download_url = dl_val.get("data").and_then(|d| d.as_str()).map(|s| s.to_string());
                                    }
                                }
                            }

                            if let Some(dl_url) = download_url {
                                if let Ok(resp) = client.get(&dl_url).send().await {
                                    if let Ok(bytes) = resp.bytes().await {
                                        crate::services::shared_storage::SharedStorageService::store_and_link("mods", filename, &bytes, &dest_path).ok();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            completed += 1;
        }

        progress_cb(ModpackInstallProgress {
            step: "¡Modpack de CurseForge instalado correctamente!".to_string(),
            current_file: manifest.name.clone(),
            completed_files: total_files,
            total_files,
            percent: 100.0,
        });

        Ok(manifest.name)
    }
}
