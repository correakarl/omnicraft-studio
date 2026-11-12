// Verificador nativo de actualizaciones de Omnicraft Launcher Studio
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_notes: String,
    pub download_url: String,
}

pub struct UpdateCheckerService;

impl UpdateCheckerService {
    pub const CURRENT_VERSION: &'static str = "1.0.0";

    pub async fn check_for_updates() -> Result<AppUpdateInfo, String> {
        // Consultar API de releases oficial en GitHub
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/1.0.0")
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let repo_url = "https://api.github.com/repos/correakarl/Omnicraft-v2/releases/latest";
        
        match client.get(repo_url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(release) = resp.json::<serde_json::Value>().await {
                        let tag_name = release["tag_name"].as_str().unwrap_or("v1.0.0");
                        let latest_clean = tag_name.trim_start_matches('v');
                        let body = release["body"].as_str().unwrap_or("Mejoras de rendimiento y estabilidad.").to_string();
                        let html_url = release["html_url"].as_str().unwrap_or("https://github.com/correakarl/Omnicraft-v2").to_string();

                        let has_update = Self::compare_semver(latest_clean, Self::CURRENT_VERSION);

                        return Ok(AppUpdateInfo {
                            current_version: format!("v{}", Self::CURRENT_VERSION),
                            latest_version: format!("v{}", latest_clean),
                            has_update,
                            release_notes: body,
                            download_url: html_url,
                        });
                    }
                }
            }
            Err(_) => {}
        }

        // Si no hay conexión o no hay release remota publicada aún
        Ok(AppUpdateInfo {
            current_version: format!("v{}", Self::CURRENT_VERSION),
            latest_version: format!("v{}", Self::CURRENT_VERSION),
            has_update: false,
            release_notes: "Estás utilizando la versión más reciente y blindada de Omnicraft Launcher Studio.".to_string(),
            download_url: "https://github.com/correakarl/Omnicraft-v2".to_string(),
        })
    }

    fn compare_semver(remote: &str, current: &str) -> bool {
        let parse = |s: &str| -> Vec<u32> {
            s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect()
        };
        let r = parse(remote);
        let c = parse(current);
        if r.len() >= 3 && c.len() >= 3 {
            if r[0] > c[0] { return true; }
            if r[0] == c[0] && r[1] > c[1] { return true; }
            if r[0] == c[0] && r[1] == c[1] && r[2] > c[2] { return true; }
        }
        false
    }
}
