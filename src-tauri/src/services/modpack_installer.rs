// Servicio de Extracción e Instalación de Modpacks (.mrpack / .zip)
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

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
    /// Descomprime overrides de un .mrpack y descarga los mods listados en modrinth.index.json
    pub async fn install_mrpack<F>(
        mrpack_path: &Path,
        minecraft_dir: &Path,
        mut progress_cb: F,
    ) -> Result<String, String>
    where
        F: FnMut(ModpackInstallProgress) + Send + 'static,
    {
        let file = File::open(mrpack_path).map_err(|e| format!("Error abriendo {}: {}", mrpack_path.display(), e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Formato mrpack zip inválido: {}", e))?;

        // 1. Leer modrinth.index.json
        let mut index_file = archive.by_name("modrinth.index.json")
            .map_err(|_| "El archivo modpack no contiene modrinth.index.json válido".to_string())?;
        
        let mut index_content = String::new();
        index_file.read_to_string(&mut index_content).map_err(|e| e.to_string())?;
        let index: MrpackIndex = serde_json::from_str(&index_content).map_err(|e| format!("Error parseando modrinth.index.json: {}", e))?;

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

        // 3. Descargar cada archivo declarado en index.files
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
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
}
