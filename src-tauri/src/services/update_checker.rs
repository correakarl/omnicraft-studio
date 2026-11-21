// Verificador nativo de actualizaciones de Omnicraft Launcher Studio
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateAsset {
    pub name: String,
    pub os_type: String,     // "linux" | "windows" | "macos"
    pub format: String,      // "deb" | "appimage" | "rpm" | "exe" | "msi" | "dmg" | "app.tar.gz"
    pub size_bytes: u64,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_notes: String,
    pub download_url: String,
    pub current_os: String,
    pub recommended_asset: Option<AppUpdateAsset>,
    pub assets: Vec<AppUpdateAsset>,
}

pub struct UpdateCheckerService;

impl UpdateCheckerService {
    pub const CURRENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");
    pub const REPO_SLUG: &'static str = "correakarl/omnicraft-studio";

    pub fn detect_current_os() -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        }
    }

    pub async fn check_for_updates() -> Result<AppUpdateInfo, String> {
        let client = reqwest::Client::builder()
            .user_agent(format!("OmnicraftStudio/{}", Self::CURRENT_VERSION))
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .map_err(|e| e.to_string())?;

        let current_os = Self::detect_current_os().to_string();
        let repo_api_url = format!("https://api.github.com/repos/{}/releases/latest", Self::REPO_SLUG);

        if let Ok(resp) = client.get(&repo_api_url).send().await {
            if resp.status().is_success() {
                if let Ok(release) = resp.json::<serde_json::Value>().await {
                    let tag_name = release["tag_name"].as_str().unwrap_or("v1.0.0");
                    let latest_clean = tag_name.trim_start_matches('v').trim();
                    let body = release["body"]
                        .as_str()
                        .unwrap_or("Nueva versión oficial disponible en el repositorio de GitHub.")
                        .to_string();
                    let html_url = release["html_url"]
                        .as_str()
                        .unwrap_or("https://github.com/correakarl/omnicraft-studio/releases")
                        .to_string();

                    let mut assets = Vec::new();
                    if let Some(raw_assets) = release["assets"].as_array() {
                        for item in raw_assets {
                            let name = item["name"].as_str().unwrap_or("").to_string();
                            let url = item["browser_download_url"].as_str().unwrap_or("").to_string();
                            let size = item["size"].as_u64().unwrap_or(0);
                            if !name.is_empty() && !url.is_empty() {
                                if let Some((os_type, format)) = Self::classify_asset(&name) {
                                    assets.push(AppUpdateAsset {
                                        name,
                                        os_type,
                                        format,
                                        size_bytes: size,
                                        download_url: url,
                                    });
                                }
                            }
                        }
                    }

                    let recommended_asset = Self::pick_recommended_asset(&assets, &current_os);
                    let has_update = Self::compare_semver(latest_clean, Self::CURRENT_VERSION);

                    return Ok(AppUpdateInfo {
                        current_version: format!("v{}", Self::CURRENT_VERSION),
                        latest_version: format!("v{}", latest_clean),
                        has_update,
                        release_notes: body,
                        download_url: html_url,
                        current_os,
                        recommended_asset,
                        assets,
                    });
                }
            }
        }

        Ok(AppUpdateInfo {
            current_version: format!("v{}", Self::CURRENT_VERSION),
            latest_version: format!("v{}", Self::CURRENT_VERSION),
            has_update: false,
            release_notes: "Estás utilizando la versión más reciente y blindada de Omnicraft Studio.".to_string(),
            download_url: format!("https://github.com/{}/releases/latest", Self::REPO_SLUG),
            current_os,
            recommended_asset: None,
            assets: Vec::new(),
        })
    }

    pub async fn download_update_installer(download_url: &str, filename: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent(format!("OmnicraftStudio/{}", Self::CURRENT_VERSION))
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

        let resp = client
            .get(download_url)
            .send()
            .await
            .map_err(|e| format!("Error conectando con el repositorio: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("El servidor devolvió estado HTTP {}", resp.status()));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Error descargando el instalador: {}", e))?;

        let target_dir = Self::resolve_downloads_dir();
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("No se pudo crear la carpeta de descargas: {}", e))?;

        let safe_name = filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect::<String>();
        let final_name = if safe_name.is_empty() {
            "Omnicraft-Studio-Update".to_string()
        } else {
            safe_name
        };

        let dest_path = target_dir.join(&final_name);
        let mut file = std::fs::File::create(&dest_path)
            .map_err(|e| format!("No se pudo guardar el archivo '{}': {}", dest_path.display(), e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Error escribiendo el instalador en disco: {}", e))?;

        #[cfg(unix)]
        if final_name.ends_with(".AppImage") {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&dest_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&dest_path, perms);
            }
        }

        Ok(dest_path.to_string_lossy().to_string())
    }

    pub fn launch_installer_and_prepare_exit(installer_path: &str) -> Result<(), String> {
        let path = PathBuf::from(installer_path);
        if !path.exists() {
            return Err(format!("El archivo instalador no existe en: {}", installer_path));
        }

        #[cfg(target_os = "windows")]
        {
            let lower = installer_path.to_lowercase();
            if lower.ends_with(".msi") {
                std::process::Command::new("msiexec")
                    .args(["/i", installer_path])
                    .spawn()
                    .map_err(|e| format!("Error iniciando msiexec: {}", e))?;
            } else {
                std::process::Command::new("cmd")
                    .args(["/C", "start", "", installer_path])
                    .spawn()
                    .map_err(|e| format!("Error ejecutando el instalador en Windows: {}", e))?;
            }
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Error abriendo el instalador en macOS: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            let lower = installer_path.to_lowercase();
            if lower.ends_with(".appimage") {
                std::process::Command::new(&path)
                    .spawn()
                    .map_err(|e| format!("Error ejecutando AppImage: {}", e))?;
            } else if let Some(parent) = path.parent() {
                std::process::Command::new("xdg-open")
                    .arg(parent)
                    .spawn()
                    .map_err(|e| format!("Error abriendo carpeta de descargas: {}", e))?;
            }
        }

        Ok(())
    }

    fn resolve_downloads_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let base = PathBuf::from(&home);
            for candidate in ["Descargas", "Downloads"] {
                let p = base.join(candidate);
                if p.exists() && p.is_dir() {
                    return p;
                }
            }
            let fallback = base.join("Downloads");
            return fallback;
        }
        PathBuf::from(".")
    }

    fn classify_asset(name: &str) -> Option<(String, String)> {
        let lower = name.to_lowercase();
        if lower.ends_with(".deb") {
            Some(("linux".to_string(), "deb".to_string()))
        } else if lower.ends_with(".appimage") {
            Some(("linux".to_string(), "appimage".to_string()))
        } else if lower.ends_with(".rpm") {
            Some(("linux".to_string(), "rpm".to_string()))
        } else if lower.ends_with(".exe") {
            Some(("windows".to_string(), "exe".to_string()))
        } else if lower.ends_with(".msi") {
            Some(("windows".to_string(), "msi".to_string()))
        } else if lower.ends_with(".dmg") {
            Some(("macos".to_string(), "dmg".to_string()))
        } else if lower.ends_with(".app.tar.gz") {
            Some(("macos".to_string(), "app.tar.gz".to_string()))
        } else {
            None
        }
    }

    fn pick_recommended_asset(assets: &[AppUpdateAsset], current_os: &str) -> Option<AppUpdateAsset> {
        let preferred_formats: &[&str] = match current_os {
            "windows" => &["exe", "msi"],
            "macos" => &["dmg", "app.tar.gz"],
            _ => &["deb", "appimage", "rpm"],
        };

        for fmt in preferred_formats {
            if let Some(found) = assets.iter().find(|a| a.os_type == current_os && a.format == *fmt) {
                return Some(found.clone());
            }
        }
        assets.iter().find(|a| a.os_type == current_os).cloned()
    }

    fn compare_semver(remote: &str, current: &str) -> bool {
        let parse = |s: &str| -> Vec<u32> {
            s.trim_start_matches('v')
                .split(|c: char| !c.is_ascii_digit())
                .filter(|p| !p.is_empty())
                .filter_map(|p| p.parse::<u32>().ok())
                .collect()
        };
        let r = parse(remote);
        let c = parse(current);
        for i in 0..3 {
            let rv = *r.get(i).unwrap_or(&0);
            let cv = *c.get(i).unwrap_or(&0);
            if rv > cv {
                return true;
            }
            if rv < cv {
                return false;
            }
        }
        false
    }
}
