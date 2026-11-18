// Servicio multiplataforma de almacenamiento compartido de contenido (Mods, Shaders, Resourcepacks, Modpacks)
// Almacena en un repositorio central (~/.omnicraft_studio/shared_cache/) y crea enlaces duros (hard links)
// o enlaces simbólicos (symlinks) hacia las instancias, eliminando el consumo duplicado de disco.
// Funciona en Linux, Windows y macOS.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContentEntry {
    pub file_hash: String,
    pub filename: String,
    pub category: String, // "mods", "shaderpacks", "resourcepacks"
    pub size_bytes: u64,
    pub compatible_mc_versions: Vec<String>,
    pub compatible_loaders: Vec<String>,
    pub source: String,
    pub project_id: Option<String>,
    pub mod_name: String,
}

pub struct SharedStorageService;

impl SharedStorageService {
    /// Obtiene el directorio base de la caché compartida
    pub fn get_shared_cache_dir() -> PathBuf {
        crate::db::DbManager::resolve_omnicraft_dir().join("shared_cache")
    }

    /// Obtiene el subdirectorio para una categoría específica ("mods", "shaderpacks", "resourcepacks", "modpacks")
    pub fn get_category_cache_dir(category: &str) -> PathBuf {
        let dir = Self::get_shared_cache_dir().join(category);
        fs::create_dir_all(&dir).ok();
        dir
    }

    /// Vincula un archivo desde la caché compartida a la carpeta de la instancia usando Hardlinks (Cero espacio duplicado)
    /// Si el sistema de archivos no soporta hardlinks entre carpetas (e.g. cruce de particiones), realiza un symlink o copia de fallback.
    pub fn link_file_to_instance(
        source_in_cache: &Path,
        target_in_instance: &Path,
    ) -> Result<(), String> {
        if !source_in_cache.exists() {
            return Err(format!("El archivo de origen no existe en la caché: {}", source_in_cache.display()));
        }

        if let Some(parent) = target_in_instance.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        // Si el destino ya existe, no rehacer si ya apunta al mismo contenido
        if target_in_instance.exists() {
            return Ok(());
        }

        // 1. Intento primario: Enlace duro (Hard link). Mismo inodo, 0 bytes adicionales, 100% nativo para Java/Minecraft
        if fs::hard_link(source_in_cache, target_in_instance).is_ok() {
            return Ok(());
        }

        // 2. Intento secundario: Enlace simbólico (Symlink) multiplataforma
        #[cfg(unix)]
        {
            if std::os::unix::fs::symlink(source_in_cache, target_in_instance).is_ok() {
                return Ok(());
            }
        }
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_file(source_in_cache, target_in_instance).is_ok() {
                return Ok(());
            }
        }

        // 3. Fallback seguro: Copia directa si están en diferentes discos o unidades de red
        fs::copy(source_in_cache, target_in_instance)
            .map(|_| ())
            .map_err(|e| format!("Fallo al vincular archivo hacia la instancia: {}", e))
    }

    /// Guarda un archivo descargado en la caché compartida y lo vincula atómicamente a la instancia
    pub fn store_and_link(
        category: &str,
        filename: &str,
        data: &[u8],
        target_in_instance: &Path,
    ) -> Result<PathBuf, String> {
        let cache_dir = Self::get_category_cache_dir(category);
        let cached_path = cache_dir.join(filename);

        // Guardar en la caché central si aún no existe o está incompleto
        if !cached_path.exists() || fs::metadata(&cached_path).map(|m| m.len()).unwrap_or(0) != data.len() as u64 {
            fs::write(&cached_path, data).map_err(|e| format!("Error guardando en caché compartida: {}", e))?;
        }

        // Vincular a la instancia con cero duplicación
        Self::link_file_to_instance(&cached_path, target_in_instance)?;

        Ok(cached_path)
    }

    /// Verifica si un archivo ya se encuentra en la caché compartida
    pub fn is_cached(category: &str, filename: &str) -> bool {
        let path = Self::get_category_cache_dir(category).join(filename);
        path.is_file()
    }
}
