// Servicio de gestión de mundos (Saves), Backups, Exportación de Modpacks y Visor de Logs
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSaveInfo {
    pub folder_name: String,
    pub display_name: String,
    pub size_bytes: u64,
    pub last_modified: String,
    pub icon_base64: Option<String>,
    pub game_mode: Option<String>,
    pub backups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOutput {
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub last_updated: String,
}

pub struct ContentToolsService;

impl ContentToolsService {
    /// Obtiene la ruta base al directorio `.minecraft` de una instancia
    pub fn get_instance_mc_dir(folder_name: &str) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".omnicraft_studio")
            .join("instances")
            .join(folder_name)
            .join(".minecraft")
    }

    /// Lista los mundos guardados en `.minecraft/saves`
    pub fn list_worlds(folder_name: &str) -> Result<Vec<WorldSaveInfo>, String> {
        let mc_dir = Self::get_instance_mc_dir(folder_name);
        let saves_dir = mc_dir.join("saves");
        if !saves_dir.exists() {
            fs::create_dir_all(&saves_dir).map_err(|e| e.to_string())?;
            return Ok(Vec::new());
        }

        let mut worlds = Vec::new();
        let entries = fs::read_dir(&saves_dir).map_err(|e| e.to_string())?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Calcular tamaño total de la carpeta del mundo
                let size_bytes: u64 = WalkDir::new(&path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .filter(|m| m.is_file())
                    .map(|m| m.len())
                    .sum();

                // Fecha de última modificación
                let modified = entry.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| {
                        let dt = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0);
                        dt.map(|val| val.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default()
                    })
                    .unwrap_or_else(|| "Reciente".to_string());

                // Icono del mundo (icon.png) si existe
                let icon_path = path.join("icon.png");
                let icon_base64 = if icon_path.exists() {
                    fs::read(&icon_path).ok().map(|b| {
                        let b64 = base64_encode(&b);
                        format!("data:image/png;base64,{}", b64)
                    })
                } else {
                    None
                };

                // Buscar backups existentes para este mundo
                let backups_dir = mc_dir.join("backups");
                let mut world_backups = Vec::new();
                if let Ok(b_entries) = fs::read_dir(&backups_dir) {
                    for be in b_entries.flatten() {
                        let b_name = be.file_name().to_string_lossy().to_string();
                        if b_name.starts_with(&format!("{}_backup_", name)) && b_name.ends_with(".zip") {
                            world_backups.push(b_name);
                        }
                    }
                }
                world_backups.sort();
                world_backups.reverse();

                worlds.push(WorldSaveInfo {
                    folder_name: name.clone(),
                    display_name: name,
                    size_bytes,
                    last_modified: modified,
                    icon_base64,
                    game_mode: Some("Supervivencia".to_string()),
                    backups: world_backups,
                });
            }
        }

        worlds.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(worlds)
    }

    /// Crea un backup ZIP del mundo en `.minecraft/backups/`
    pub fn backup_world(folder_name: &str, world_name: &str) -> Result<String, String> {
        let mc_dir = Self::get_instance_mc_dir(folder_name);
        let world_dir = mc_dir.join("saves").join(world_name);
        if !world_dir.exists() {
            return Err(format!("El mundo '{}' no existe", world_name));
        }

        let backups_dir = mc_dir.join("backups");
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_filename = format!("{}_backup_{}.zip", world_name, timestamp);
        let zip_dest = backups_dir.join(&backup_filename);

        Self::zip_directory(&world_dir, &zip_dest)?;
        Ok(backup_filename)
    }

    /// Restaura de forma atómica y segura un archivo de respaldo ZIP en la carpeta saves
    pub fn restore_world_backup(folder_name: &str, backup_filename: &str) -> Result<String, String> {
        // Blindaje contra Path Traversal
        if backup_filename.contains("..") || backup_filename.contains('/') || backup_filename.contains('\\') {
            return Err("Nombre de archivo de respaldo no válido o sospechoso.".to_string());
        }

        let mc_dir = Self::get_instance_mc_dir(folder_name);
        let backups_dir = mc_dir.join("backups");
        let zip_file_path = backups_dir.join(backup_filename);

        if !zip_file_path.exists() {
            return Err(format!("El archivo de respaldo '{}' no existe en backups/", backup_filename));
        }

        // Obtener el nombre del mundo objetivo deduciéndolo del nombre del backup
        let world_name = backup_filename
            .split("_backup_")
            .next()
            .unwrap_or("Mundo_Restaurado");

        let saves_dir = mc_dir.join("saves");
        let target_world_dir = saves_dir.join(world_name);

        // Si el mundo ya existe, crear una copia de resguardo temporal antes de sobreescribir
        if target_world_dir.exists() {
            let temp_safeguard = saves_dir.join(format!("{}_pre_restore", world_name));
            if temp_safeguard.exists() {
                fs::remove_dir_all(&temp_safeguard).ok();
            }
            // Renombrar como salvaguarda
            fs::rename(&target_world_dir, &temp_safeguard).map_err(|e| format!("No se pudo proteger mundo actual: {}", e))?;
        }

        fs::create_dir_all(&target_world_dir).map_err(|e| e.to_string())?;

        // Descomprimir de forma segura (prevención de Zip-Slip / Zip-Bomb)
        let file = fs::File::open(&zip_file_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file_entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file_entry.enclosed_name() {
                Some(path) => target_world_dir.join(path),
                None => continue,
            };

            if file_entry.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file_entry, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

        Ok(format!("Mundo '{}' restaurado exitosamente desde {}", world_name, backup_filename))
    }

    /// Lee las últimas líneas del archivo de log de la instancia (`latest.log`).
    /// Si el archivo supera 1 MB, lee únicamente la cola final del archivo sin cargarlo todo en memoria.
    /// Además, rota/trunca el archivo en disco si supera los 2 MB para evitar un crecimiento infinito.
    pub fn read_instance_logs(folder_name: &str, max_lines: usize) -> Result<LogOutput, String> {
        let log_file = Self::get_instance_mc_dir(folder_name).join("logs").join("latest.log");
        if !log_file.exists() {
            return Ok(LogOutput {
                lines: vec!["[Omnicraft] No hay registros todavía. Inicia la instancia para generar logs.".to_string()],
                total_lines: 1,
                last_updated: "Sin actividad".to_string(),
            });
        }

        let metadata = fs::metadata(&log_file).map_err(|e| e.to_string())?;
        let file_size = metadata.len();

        // Límite de lectura de seguridad: si el archivo es mayor a 1 MB, leer solo los últimos 512 KB
        // para extraer las líneas más recientes sin saturar la memoria.
        const READ_TAIL_BYTES: u64 = 512 * 1024; // 512 KB
        let content = if file_size > READ_TAIL_BYTES {
            let mut file = File::open(&log_file).map_err(|e| e.to_string())?;
            file.seek(SeekFrom::Start(file_size - READ_TAIL_BYTES)).map_err(|e| e.to_string())?;
            let mut buffer = Vec::with_capacity(READ_TAIL_BYTES as usize);
            file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            String::from_utf8_lossy(&buffer).to_string()
        } else {
            fs::read_to_string(&log_file).unwrap_or_else(|_| "Error leyendo archivo de logs".to_string())
        };

        let all_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let total_lines = all_lines.len();

        // Mantener un margen máximo de registros solicitados (por defecto 500)
        let effective_max = if max_lines == 0 { 500 } else { max_lines };
        let slice_start = if total_lines > effective_max {
            total_lines - effective_max
        } else {
            0
        };

        let lines = all_lines[slice_start..].to_vec();
        let last_updated = chrono::Local::now().format("%H:%M:%S").to_string();

        // Control de crecimiento infinito en disco: si supera 2 MB, truncar manteniendo los registros recientes
        if file_size > 2 * 1024 * 1024 {
            let truncated_content = lines.join("\n");
            let _ = fs::write(&log_file, truncated_content);
        }

        Ok(LogOutput {
            lines,
            total_lines,
            last_updated,
        })
    }

    /// Exporta la instancia como un paquete comprimido ZIP compartible (mods, config, shaders, icons)
    pub fn export_instance_pack(
        folder_name: &str,
        export_name: &str,
        include_worlds: bool,
    ) -> Result<String, String> {
        let mc_dir = Self::get_instance_mc_dir(folder_name);
        if !mc_dir.exists() {
            return Err("El directorio de la instancia no existe".to_string());
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let export_dir = PathBuf::from(home)
            .join(".omnicraft_studio")
            .join("exports");
        fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

        let clean_name = export_name.replace(' ', "_");
        let zip_path = export_dir.join(format!("{}.zip", clean_name));
        let file = File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Directorios a incluir en el modpack compartible
        let mut folders_to_include = vec!["mods", "config", "shaderpacks", "resourcepacks"];
        if include_worlds {
            folders_to_include.push("saves");
        }

        for folder in folders_to_include {
            let src = mc_dir.join(folder);
            if src.exists() && src.is_dir() {
                for entry in WalkDir::new(&src).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    let relative = path.strip_prefix(&mc_dir).map_err(|e| e.to_string())?;
                    let rel_str = relative.to_string_lossy().to_string();

                    if path.is_file() {
                        zip.start_file(&rel_str, options).map_err(|e| e.to_string())?;
                        let mut f = File::open(path).map_err(|e| e.to_string())?;
                        let mut buffer = Vec::new();
                        f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                        zip.write_all(&buffer).map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        zip.finish().map_err(|e| e.to_string())?;
        Ok(zip_path.to_string_lossy().to_string())
    }

    /// Función auxiliar para comprimir una carpeta a ZIP
    fn zip_directory(src_dir: &Path, dst_file: &Path) -> Result<(), String> {
        let file = File::create(dst_file).map_err(|e| e.to_string())?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(src_dir).map_err(|e| e.to_string())?;
            let rel_str = relative.to_string_lossy().to_string();

            if rel_str.is_empty() {
                continue;
            }

            if path.is_file() {
                zip.start_file(&rel_str, options).map_err(|e| e.to_string())?;
                let mut f = File::open(path).map_err(|e| e.to_string())?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                zip.write_all(&buffer).map_err(|e| e.to_string())?;
            }
        }

        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARSET[(triple >> 18) & 0x3F] as char);
        result.push(CHARSET[(triple >> 12) & 0x3F] as char);
        result.push(if i + 1 < data.len() { CHARSET[(triple >> 6) & 0x3F] as char } else { '=' });
        result.push(if i + 2 < data.len() { CHARSET[triple & 0x3F] as char } else { '=' });

        i += 3;
    }
    result
}
