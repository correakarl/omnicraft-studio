// Cliente de búsqueda en tiempo real hacia la API oficial de Modrinth
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub loaders: Vec<String>,
    pub versions: Vec<String>,
}

pub struct ModrinthClient;

impl ModrinthClient {
    pub async fn search(
        query: &str,
        project_type: &str, // "mod", "shader", "resourcepack", "modpack"
        mc_version: &str,
        loader: &str,
        limit: u32,
    ) -> Result<Vec<ModSearchResult>, String> {
        let mut facets = Vec::new();

        if !project_type.is_empty() {
            facets.push(format!("[\"project_type:{}\"]", project_type));
        }

        if !mc_version.is_empty() && mc_version != "Todas" {
            facets.push(format!("[\"versions:{}\"]", mc_version));
        }

        if !loader.is_empty() && loader != "Todos" && loader != "General" && loader != "Vanilla" {
            facets.push(format!("[\"categories:{}\"]", loader.to_lowercase()));
        }

        let facets_str = if facets.is_empty() {
            String::new()
        } else {
            format!("[{}]", facets.join(","))
        };

        let mut url = format!(
            "https://api.modrinth.com/v2/search?query={}&limit={}&index=downloads",
            urlencoding::encode(query),
            limit
        );

        if !facets_str.is_empty() {
            url.push_str(&format!("&facets={}", urlencoding::encode(&facets_str)));
        }

        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Error HTTP de Modrinth: {}", res.status()));
        }

        let val: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        let hits = val.get("hits").and_then(|h| h.as_array()).cloned().unwrap_or_default();

        let mut results = Vec::new();
        for h in hits {
            let id = h.get("project_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let slug = h.get("slug").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let title = h.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let description = h.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let author = h.get("author").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let downloads = h.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);
            let icon_url = h.get("icon_url").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ptype = h.get("project_type").and_then(|v| v.as_str()).unwrap_or("mod").to_string();

            let loaders: Vec<String> = h.get("categories")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let versions: Vec<String> = h.get("versions")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            results.push(ModSearchResult {
                id,
                slug,
                title,
                description,
                author,
                downloads,
                icon_url,
                project_type: ptype,
                loaders,
                versions,
            });
        }

        Ok(results)
    }
}
