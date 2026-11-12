// Detector y resolutor nativo de ejecutables de Java en Linux, Windows y macOS
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntime {
    pub path: String,
    pub version: u32,
    pub is_64bit: bool,
    pub vendor: String,
}

pub struct JavaService;

impl JavaService {
    pub fn detect_all_javas() -> Vec<JavaRuntime> {
        let mut found = Vec::new();
        let home = std::env::var("HOME").unwrap_or_default();
        
        // Rutas habituales en Linux y macOS
        let candidate_dirs = vec![
            "/usr/lib/jvm".to_string(),
            "/usr/lib64/jvm".to_string(),
            "/opt/java".to_string(),
            format!("{}/.omnicraft_studio/runtimes", home),
            format!("{}/Games/MultiMinecraft_Heberon/runtime", home),
            format!("{}/Games/Omnicraft-v2/runtime", home),
            format!("{}/.minecraft/runtime", home),
            format!("{}/.local/share/runtimes", home),
        ];

        // 1. Java en PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                let bin = Path::new(dir).join("java");
                if bin.is_file() {
                    if let Some(rt) = Self::inspect_java(&bin) {
                        if !found.iter().any(|j: &JavaRuntime| j.path == rt.path) {
                            found.push(rt);
                        }
                    }
                }
            }
        }

        // 2. Buscar en directorios estándar de JVM
        for cdir in candidate_dirs {
            let p = Path::new(&cdir);
            if p.is_dir() {
                if let Ok(entries) = std::fs::read_dir(p) {
                    for entry in entries.flatten() {
                        let bin = entry.path().join("bin").join("java");
                        if bin.is_file() {
                            if let Some(rt) = Self::inspect_java(&bin) {
                                if !found.iter().any(|j: &JavaRuntime| j.path == rt.path) {
                                    found.push(rt);
                                }
                            }
                        }
                    }
                }
            }
        }

        found
    }

    pub fn inspect_java(java_bin: &Path) -> Option<JavaRuntime> {
        let out = Command::new(java_bin)
            .arg("-version")
            .output()
            .ok()?;

        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let full = format!("{}\n{}", stdout, stderr);

        let mut ver = 8;
        if full.contains("1.8.") {
            ver = 8;
        } else {
            for v in (9..=25).rev() {
                if full.contains(&format!("\"{}\"", v)) || full.contains(&format!("version \"{}.", v)) || full.contains(&format!("openjdk {}", v)) {
                    ver = v;
                    break;
                }
            }
        }

        let is_64 = full.contains("64-Bit") || full.contains("x86_64") || full.contains("aarch64");
        let vendor = if full.contains("Adoptium") || full.contains("Temurin") {
            "Eclipse Temurin".to_string()
        } else if full.contains("Zulu") {
            "Azul Zulu".to_string()
        } else if full.contains("OpenJDK") {
            "OpenJDK".to_string()
        } else {
            "Oracle / Estándar".to_string()
        };

        Some(JavaRuntime {
            path: java_bin.to_string_lossy().to_string(),
            version: ver,
            is_64bit: is_64,
            vendor,
        })
    }

    pub fn recommend_java_for_version(mc_version: &str) -> u32 {
        if mc_version.starts_with("1.21") || mc_version.starts_with("1.20.5") || mc_version.starts_with("1.20.6") {
            21 // Java 21 para 1.20.5+ y 1.21+
        } else if mc_version.starts_with("1.18") || mc_version.starts_with("1.19") || mc_version.starts_with("1.20") {
            17 // Java 17 para 1.18 a 1.20.4
        } else if mc_version.starts_with("1.17") {
            16
        } else {
            8 // Java 8 para 1.16.5 y anteriores
        }
    }
}
