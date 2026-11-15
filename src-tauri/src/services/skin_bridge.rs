// Puente de Skins: Inyección nativa offline (CustomSkinLoader / Ely.by)
// y migración hacia servidores oficiales de Mojang para cuentas Premium
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePublicSkin {
    pub id: String,
    pub name: String,
    pub source: String,
    pub texture_url: String,
    pub preview_url: String,
    pub model_type: String,
}

pub struct SkinBridgeService;

impl SkinBridgeService {
    /// Inyecta la skin en la instancia de juego para jugadores Non-Premium (Offline)
    /// mediante la estructura estándar de CustomSkinLoader (.minecraft/CustomSkinLoader/)
    pub fn inject_offline_skin(
        mc_dir: &Path,
        username: &str,
        base64_png: &str,
    ) -> Result<(), String> {
        let csl_dir = mc_dir.join("CustomSkinLoader");
        let local_skin_dir = csl_dir.join("LocalSkin").join("skins");
        fs::create_dir_all(&local_skin_dir).map_err(|e| e.to_string())?;

        // 1. Decodificar Base64 y guardar el archivo PNG con el nombre del usuario
        let clean_base64 = if base64_png.contains(',') {
            base64_png.split(',').nth(1).unwrap_or(base64_png)
        } else {
            base64_png
        };

        // Decodificación manual o guardado directo
        if let Ok(bytes) = Self::decode_base64(clean_base64) {
            let target_png = local_skin_dir.join(format!("{}.png", username));
            fs::write(target_png, bytes).map_err(|e| e.to_string())?;
        }

        // 2. Generar CustomSkinLoader.json para priorizar archivos locales
        let config_file = csl_dir.join("CustomSkinLoader.json");
        let config_json = json!({
            "version": "14.20",
            "enableCache": true,
            "loadlist": [
                {
                    "name": "OmnicraftLocalSkin",
                    "type": "Legacy",
                    "skin": "file://CustomSkinLoader/LocalSkin/skins/{username}.png",
                    "cape": "file://CustomSkinLoader/LocalSkin/capes/{username}.png",
                    "elytra": "file://CustomSkinLoader/LocalSkin/elytras/{username}.png"
                },
                {
                    "name": "MojangAPI",
                    "type": "MojangAPI"
                }
            ]
        });

        fs::write(config_file, serde_json::to_string_pretty(&config_json).unwrap_or_default()).ok();

        // 3. Si la instancia tiene SkinShuffle instalado, guardar también el PNG en config/skinshuffle/skins/<username>.png
        // y actualizar presets-<username>.json para que SkinShuffle renderice esta skin local de inmediato
        let ss_dir = mc_dir.join("config").join("skinshuffle");
        if ss_dir.exists() {
            let ss_skins = ss_dir.join("skins");
            fs::create_dir_all(&ss_skins).ok();
            if let Ok(bytes) = Self::decode_base64(clean_base64) {
                let _ = fs::write(ss_skins.join(format!("{}.png", username)), bytes);
            }
            let preset_payload = json!({
                "chosenPreset": 0,
                "apiPreset": 0,
                "loadedPresets": [
                    {
                        "skin": {
                            "skin_name": username,
                            "model": "default",
                            "type": "skinshuffle:config"
                        },
                        "name": username
                    }
                ]
            });
            if let Ok(pretty) = serde_json::to_string_pretty(&preset_payload) {
                fs::write(ss_dir.join(format!("presets-{}.json", username)), &pretty).ok();
                fs::write(ss_dir.join("presets.json"), &pretty).ok();
            }
        }

        Ok(())
    }

    /// Sube la textura oficial a los servidores de Mojang para cuentas Premium
    pub async fn upload_skin_to_mojang(
        bearer_token: &str,
        base64_png: &str,
        model_type: &str, // "classic" | "slim"
    ) -> Result<(), String> {
        let clean_base64 = if base64_png.contains(',') {
            base64_png.split(',').nth(1).unwrap_or(base64_png)
        } else {
            base64_png
        };

        let bytes = Self::decode_base64(clean_base64).map_err(|e| format!("Base64 inválido: {}", e))?;

        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        let form = reqwest::multipart::Form::new()
            .text("variant", model_type.to_string())
            .part(
                "file",
                reqwest::multipart::Part::bytes(bytes)
                    .file_name("skin.png")
                    .mime_str("image/png")
                    .map_err(|e| e.to_string())?,
            );

        let res = client
            .post("https://api.minecraftservices.com/minecraft/profile/skins")
            .header("Authorization", format!("Bearer {}", bearer_token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("Mojang API respondió {}", res.status()))
        }
    }

    /// Obtiene la skin oficial de Mojang de un usuario o UUID en base64 junto con su modelo ('classic' o 'slim')
    pub async fn fetch_mojang_skin(username_or_uuid: &str) -> Result<(String, String), String> {
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .build()
            .map_err(|e| e.to_string())?;

        // 1. Obtener UUID si es un nombre de usuario
        let uuid = if username_or_uuid.len() >= 32 {
            username_or_uuid.replace("-", "")
        } else {
            let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username_or_uuid);
            let resp = client.get(&profile_url).send().await.map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!("Perfil Mojang no encontrado ({})", resp.status()));
            }
            let profile: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            profile["id"].as_str().ok_or_else(|| "ID de perfil no encontrado".to_string())?.to_string()
        };

        // 2. Obtener Perfil de sesión con las propiedades de textura
        let session_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", uuid);
        let resp = client.get(&session_url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Sesión de Mojang fallida ({})", resp.status()));
        }
        let session: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        let mut skin_url: Option<String> = None;
        let mut model_type = "classic".to_string();

        if let Some(props) = session["properties"].as_array() {
            for prop in props {
                if prop["name"].as_str() == Some("textures") {
                    if let Some(val_b64) = prop["value"].as_str() {
                        if let Ok(decoded) = Self::decode_base64(val_b64) {
                            if let Ok(text_json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                                if let Some(skin_obj) = text_json.pointer("/textures/SKIN") {
                                    skin_url = skin_obj["url"].as_str().map(|s| s.to_string());
                                    if skin_obj.pointer("/metadata/model").and_then(|m| m.as_str()) == Some("slim") {
                                        model_type = "slim".to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let skin_url = skin_url.ok_or_else(|| "El usuario no tiene una skin personalizada de Mojang".to_string())?;

        // 3. Descargar la textura PNG
        let tex_resp = client.get(&skin_url).send().await.map_err(|e| e.to_string())?;
        if !tex_resp.status().is_success() {
            return Err(format!("Error descargando textura ({})", tex_resp.status()));
        }
        let bytes = tex_resp.bytes().await.map_err(|e| e.to_string())?;

        // Convertir bytes a base64
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut b64 = String::with_capacity(bytes.len() * 4 / 3 + 4);
        for chunk in bytes.chunks(3) {
            let mut buf = [0u8; 4];
            let b0 = chunk[0];
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);

            buf[0] = TABLE[(b0 >> 2) as usize];
            buf[1] = TABLE[(((b0 & 3) << 4) | (b1 >> 4)) as usize];
            buf[2] = if chunk.len() > 1 { TABLE[(((b1 & 15) << 2) | (b2 >> 6)) as usize] } else { b'=' };
            buf[3] = if chunk.len() > 2 { TABLE[(b2 & 63) as usize] } else { b'=' };

            b64.push_str(std::str::from_utf8(&buf).unwrap_or(""));
        }

        let data_uri = format!("data:image/png;base64,{}", b64);
        Ok((data_uri, model_type))
    }

    /// Busca skins en las bibliotecas remotas públicas (MineSkin Public Registry + Perfiles Oficiales Mojang / Ely.by)
    pub async fn search_remote_public_skins(query: &str, page: u32) -> Result<Vec<RemotePublicSkin>, String> {
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let clean_q = query.trim();
        let page_num = if page == 0 { 1 } else { page };
        let mut results: Vec<RemotePublicSkin> = Vec::new();

        // 1. Si hay término de búsqueda y estamos en la página 1, consultar también si existe un jugador oficial de Mojang con ese nombre
        if !clean_q.is_empty() && page_num == 1 && !clean_q.contains(' ') && clean_q.len() <= 36 {
            let mojang_url = if clean_q.len() >= 32 {
                format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", clean_q.replace('-', ""))
            } else {
                format!("https://api.mojang.com/users/profiles/minecraft/{}", clean_q)
            };

            if let Ok(resp) = client.get(&mojang_url).send().await {
                if resp.status().is_success() {
                    if let Ok(prof) = resp.json::<serde_json::Value>().await {
                        let uname = prof["name"].as_str().unwrap_or(clean_q).to_string();
                        let uid = prof["id"].as_str().unwrap_or("").to_string();
                        if !uid.is_empty() {
                            results.push(RemotePublicSkin {
                                id: format!("mojang_{}", uid),
                                name: format!("{} (Perfil Mojang Oficial)", uname),
                                source: "Mojang / MC-Heads".to_string(),
                                texture_url: format!("https://mc-heads.net/skin/{}", uid),
                                preview_url: format!("https://mc-heads.net/body/{}/140", uid),
                                model_type: "classic".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // 2. Consultar catálogo público de MineSkin (más de 7 millones de skins comunitarias alojadas en textures.minecraft.net)
        let mineskin_url = if clean_q.is_empty() {
            format!("https://api.mineskin.org/get/list/{}?size=24", page_num)
        } else {
            // Codificar espacios y caracteres especiales de forma segura
            let encoded_q: String = clean_q
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c.to_string() } else { format!("%{:02X}", c as u8) })
                .collect();
            format!("https://api.mineskin.org/get/list/{}?size=24&filter={}", page_num, encoded_q)
        };

        if let Ok(ms_resp) = client.get(&mineskin_url).send().await {
            if ms_resp.status().is_success() {
                if let Ok(ms_json) = ms_resp.json::<serde_json::Value>().await {
                    if let Some(skins_arr) = ms_json["skins"].as_array() {
                        for item in skins_arr {
                            let id_num = item["id"].as_u64().unwrap_or(0);
                            let skin_uuid = item["uuid"]
                                .as_str()
                                .or_else(|| item["skinUuid"].as_str())
                                .unwrap_or("")
                                .to_string();
                            let raw_name = item["name"].as_str().unwrap_or("").trim();
                            let display_name = if !raw_name.is_empty() {
                                raw_name.to_string()
                            } else if !clean_q.is_empty() {
                                format!("{} #{}", clean_q, id_num % 10000)
                            } else {
                                format!("Skin Comunitaria #{}", id_num % 100000)
                            };

                            if let Some(url) = item["url"].as_str() {
                                let https_url = url.replace("http://textures.minecraft.net/", "https://textures.minecraft.net/");
                                let tex_hash = https_url.rsplit('/').next().unwrap_or(&skin_uuid);
                                let preview = format!("https://mc-heads.net/body/{}/140", tex_hash);

                                results.push(RemotePublicSkin {
                                    id: format!("ms_{}", if !skin_uuid.is_empty() { skin_uuid } else { id_num.to_string() }),
                                    name: display_name,
                                    source: "MineSkin Public".to_string(),
                                    texture_url: https_url,
                                    preview_url: preview,
                                    model_type: "classic".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Descarga cualquier textura remota (URL de textures.minecraft.net, mc-heads, etc.) y la convierte a data:image/png;base64
    pub async fn download_remote_skin_base64(texture_url: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/2.0")
            .timeout(std::time::Duration::from_secs(12))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .get(texture_url)
            .send()
            .await
            .map_err(|e| format!("Error conectando al servidor de skins: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("El servidor remoto respondió HTTP {}", resp.status()));
        }

        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut b64 = String::with_capacity(bytes.len() * 4 / 3 + 4);
        for chunk in bytes.chunks(3) {
            let mut buf = [0u8; 4];
            let b0 = chunk[0];
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);

            buf[0] = TABLE[(b0 >> 2) as usize];
            buf[1] = TABLE[(((b0 & 3) << 4) | (b1 >> 4)) as usize];
            buf[2] = if chunk.len() > 1 { TABLE[(((b1 & 15) << 2) | (b2 >> 6)) as usize] } else { b'=' };
            buf[3] = if chunk.len() > 2 { TABLE[(b2 & 63) as usize] } else { b'=' };

            b64.push_str(std::str::from_utf8(&buf).unwrap_or(""));
        }

        Ok(format!("data:image/png;base64,{}", b64))
    }

    fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
        // Decodificador RFC 4648 estándar
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut map = [255u8; 256];
        for (i, &b) in TABLE.iter().enumerate() {
            map[b as usize] = i as u8;
        }

        let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace() && *b != b'=').collect();
        let mut out = Vec::with_capacity(clean.len() * 3 / 4);

        for chunk in clean.chunks(4) {
            let mut val = 0u32;
            let mut pad = 0;
            for (i, &b) in chunk.iter().enumerate() {
                let v = map[b as usize];
                if v == 255 {
                    return Err("Carácter base64 no válido".to_string());
                }
                val |= (v as u32) << (18 - i * 6);
            }
            if chunk.len() < 4 {
                pad = 4 - chunk.len();
            }

            if pad <= 2 {
                out.push((val >> 16) as u8);
            }
            if pad <= 1 {
                out.push((val >> 8) as u8);
            }
            if pad == 0 && chunk.len() == 4 {
                out.push(val as u8);
            }
        }

        Ok(out)
    }
}
