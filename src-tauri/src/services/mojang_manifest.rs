// Manifiesto de Versiones Oficiales de Mojang (Piston Meta v2)
// Soporta todas las versiones: Releases (103), Snapshots (750+), Betas y Alphas
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersionEntry {
    pub id: String,
    pub version_type: String, // "release", "snapshot", "old_beta", "old_alpha"
    pub release_time: Option<String>,
}

pub struct MojangManifestService;

impl MojangManifestService {
    pub async fn fetch_all_versions() -> Result<Vec<GameVersionEntry>, String> {
        let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Piston Meta HTTP {}", res.status()));
        }

        let val: Value = res.json().await.map_err(|e| e.to_string())?;
        let versions = val.get("versions").and_then(|v| v.as_array()).ok_or("Formato de manifiesto inválido")?;

        let mut list = Vec::new();
        for v in versions {
            let id = v.get("id").and_then(|s| s.as_str()).unwrap_or_default().to_string();
            let vtype = v.get("type").and_then(|s| s.as_str()).unwrap_or("release").to_string();
            let release_time = v.get("releaseTime").and_then(|s| s.as_str()).map(|s| s.to_string());
            if !id.is_empty() {
                list.push(GameVersionEntry {
                    id,
                    version_type: vtype,
                    release_time,
                });
            }
        }

        Ok(list)
    }
}
