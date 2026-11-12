// Servicio de gestión y conmutación de estado de contenido (Mods, Shaders, Resourcepacks)
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub name: String,
    pub filename: String,
    pub category: String, // "mods", "shaderpacks", "resourcepacks"
    pub enabled: bool,
    pub size_bytes: u64,
}

pub struct ContentManagerService;

impl ContentManagerService {
    fn get_dir(folder_name: &str, category: &str) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".omnicraft_studio")
            .join("instances")
            .join(folder_name)
            .join(".minecraft")
            .join(category)
    }

    /// Lista todos los elementos en una categoría (mods, shaderpacks, resourcepacks)
    pub fn list_category_items(folder_name: &str, category: &str) -> Result<Vec<ContentItem>, String> {
        let dir = Self::get_dir(folder_name, category);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with('.') || file_name.ends_with(".tmp") {
                    continue;
                }

                let is_disabled = file_name.ends_with(".disabled");
                let clean_name = if is_disabled {
                    file_name.trim_end_matches(".disabled").to_string()
                } else {
                    file_name.clone()
                };

                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

                items.push(ContentItem {
                    name: clean_name,
                    filename: file_name,
                    category: category.to_string(),
                    enabled: !is_disabled,
                    size_bytes,
                });
            }
        }

        items.sort_by(|a, b| b.enabled.cmp(&a.enabled).then(a.name.cmp(&b.name)));
        Ok(items)
    }

    /// Conmuta el estado habilitado/deshabilitado de un archivo añadiendo o quitando la extensión .disabled
    pub fn toggle_item_state(folder_name: &str, category: &str, filename: &str, enable: bool) -> Result<String, String> {
        let dir = Self::get_dir(folder_name, category);
        let src_path = dir.join(filename);

        if !src_path.exists() {
            return Err(format!("El archivo '{}' no existe", filename));
        }

        let new_filename = if enable {
            // Quitar .disabled
            if filename.ends_with(".disabled") {
                filename.trim_end_matches(".disabled").to_string()
            } else {
                filename.to_string()
            }
        } else {
            // Añadir .disabled
            if !filename.ends_with(".disabled") {
                format!("{}.disabled", filename)
            } else {
                filename.to_string()
            }
        };

        let dst_path = dir.join(&new_filename);
        if src_path != dst_path {
            fs::rename(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }

        Ok(new_filename)
    }

    /// Elimina un archivo de contenido permanentemente
    pub fn delete_item(folder_name: &str, category: &str, filename: &str) -> Result<(), String> {
        let path = Self::get_dir(folder_name, category).join(filename);
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(path).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
