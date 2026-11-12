// Verificador de Compatibilidad de Mods & Modloaders en Rust nativo
use std::path::Path;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub filename: String,
    pub mod_id: String,
    pub name: String,
    pub version: String,
    pub detected_loader: String, // "fabric" | "forge" | "neoforge" | "quilt" | "unknown"
    pub mc_dependencies: Vec<String>,
    pub required_dependencies: Vec<String>,
    pub incompatible_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub is_compatible: bool,
    pub checked_mods_count: usize,
    pub instance_mc_version: String,
    pub instance_loader: String,
    pub issues: Vec<CompatibilityIssue>,
    pub mods: Vec<ModMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    pub severity: String, // "error" | "warning" | "info"
    pub mod_filename: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
}

pub struct CompatibilityCheckerService;

impl CompatibilityCheckerService {
    /// Inspecciona el directorio .minecraft/mods y valida que los mods coincidan con la versión de Minecraft y ModLoader
    pub fn check_instance_mods(
        minecraft_dir: &Path,
        instance_mc_version: &str,
        instance_loader: &str,
    ) -> CompatibilityReport {
        let mods_dir = minecraft_dir.join("mods");
        let mut report = CompatibilityReport {
            is_compatible: true,
            checked_mods_count: 0,
            instance_mc_version: instance_mc_version.to_string(),
            instance_loader: instance_loader.to_string(),
            issues: Vec::new(),
            mods: Vec::new(),
        };

        if !mods_dir.exists() {
            return report;
        }

        let entries = match std::fs::read_dir(&mods_dir) {
            Ok(e) => e,
            Err(_) => return report,
        };

        let mut installed_mod_ids = std::collections::HashSet::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let meta = Self::inspect_jar(&path, &filename);
                if !meta.mod_id.is_empty() {
                    installed_mod_ids.insert(meta.mod_id.clone());
                }
                report.mods.push(meta);
            }
        }

        report.checked_mods_count = report.mods.len();

        let inst_loader_lower = instance_loader.to_lowercase();

        // Validar compatibilidad de cada mod
        for m in &report.mods {
            let loader_lower = m.detected_loader.to_lowercase();
            
            // 1. Validar compatibilidad de Loader (ej. mod Forge en instancia Fabric)
            if loader_lower != "unknown" {
                let compatible = match inst_loader_lower.as_str() {
                    "fabric" => loader_lower == "fabric" || loader_lower == "quilt",
                    "quilt" => loader_lower == "fabric" || loader_lower == "quilt",
                    "forge" => loader_lower == "forge",
                    "neoforge" => loader_lower == "neoforge" || loader_lower == "forge",
                    _ => true,
                };

                if !compatible {
                    report.is_compatible = false;
                    report.issues.push(CompatibilityIssue {
                        severity: "error".to_string(),
                        mod_filename: m.filename.clone(),
                        title: format!("Incompatibilidad de Modloader: {}", m.name),
                        description: format!(
                            "El mod '{}' fue diseñado para {}, pero la instancia usa {}.",
                            m.name, m.detected_loader, instance_loader
                        ),
                        recommendation: format!(
                            "Descarga la versión compilada para {} o cambia el loader de la instancia.",
                            instance_loader
                        ),
                    });
                }
            }

            // 2. Comprobar dependencias faltantes
            for dep in &m.required_dependencies {
                if dep == "fabricloader" || dep == "forge" || dep == "minecraft" || dep == "java" {
                    continue;
                }
                // Si requiere fabric y no está fabric-api ni fabric
                if dep == "fabric" || dep == "fabric-api" {
                    if !installed_mod_ids.contains("fabric-api") && !installed_mod_ids.contains("fabric") {
                        report.issues.push(CompatibilityIssue {
                            severity: "warning".to_string(),
                            mod_filename: m.filename.clone(),
                            title: format!("Dependencia requerida: {}", dep),
                            description: format!("El mod '{}' requiere '{}' para funcionar sin crasheos.", m.name, dep),
                            recommendation: "Instala Fabric API desde la Tienda de Mods.".to_string(),
                        });
                    }
                }
            }
        }

        report
    }

    fn inspect_jar(jar_path: &Path, filename: &str) -> ModMetadata {
        let mut meta = ModMetadata {
            filename: filename.to_string(),
            mod_id: String::new(),
            name: filename.replace(".jar", ""),
            version: String::new(),
            detected_loader: "unknown".to_string(),
            mc_dependencies: Vec::new(),
            required_dependencies: Vec::new(),
            incompatible_dependencies: Vec::new(),
        };

        let file = match File::open(jar_path) {
            Ok(f) => f,
            Err(_) => return meta,
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return meta,
        };

        // Comprobar Fabric (fabric.mod.json)
        if let Ok(mut f) = archive.by_name("fabric.mod.json") {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                    meta.detected_loader = "Fabric".to_string();
                    meta.mod_id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    meta.name = val.get("name").and_then(|v| v.as_str()).unwrap_or(filename).to_string();
                    meta.version = val.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    
                    if let Some(depends) = val.get("depends").and_then(|v| v.as_object()) {
                        for (k, _) in depends {
                            meta.required_dependencies.push(k.clone());
                        }
                    }
                    return meta;
                }
            }
        }

        // Comprobar Forge / NeoForge (META-INF/mods.toml)
        if let Ok(mut f) = archive.by_name("META-INF/mods.toml") {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                meta.detected_loader = if s.contains("neoforge") { "NeoForge".to_string() } else { "Forge".to_string() };
                meta.mod_id = filename.to_string();
                return meta;
            }
        }

        // Comprobar Legacy Forge (mcmod.info)
        if let Ok(mut f) = archive.by_name("mcmod.info") {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                meta.detected_loader = "Forge".to_string();
                return meta;
            }
        }

        // Detección heurística por nombre de archivo
        let f_lower = filename.to_lowercase();
        if f_lower.contains("fabric") {
            meta.detected_loader = "Fabric".to_string();
        } else if f_lower.contains("neoforge") {
            meta.detected_loader = "NeoForge".to_string();
        } else if f_lower.contains("forge") {
            meta.detected_loader = "Forge".to_string();
        }

        meta
    }
}
