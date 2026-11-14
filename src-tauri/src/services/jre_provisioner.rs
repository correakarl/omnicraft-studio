// Servicio de descarga y aprovisionamiento automático de Java OpenJDK (Adoptium / Temurin)
// y autenticación nativa Device Code Flow oficial de Microsoft / Xbox Live / Mojang
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftDeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftAccountSession {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub skin_url: Option<String>,
}

pub struct JreProvisionerService;

impl JreProvisionerService {
    /// Descarga e instala automáticamente el runtime JRE de Temurin (Adoptium) para la versión solicitada (8, 17 o 21)
    pub async fn install_temurin_jre(major_version: u32) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent("OmnicraftLauncherStudio/1.0.0")
            .build()
            .map_err(|e| e.to_string())?;

        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/linux/x64/jre/hotspot/normal/eclipse?project=jdk",
            major_version
        );

        let resp = client.get(&url).send().await.map_err(|e| format!("Error conectando con Adoptium: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Adoptium API retornó {}", resp.status()));
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut runtimes_dir = PathBuf::from(&home);
        runtimes_dir.push(".omnicraft_studio");
        runtimes_dir.push("runtimes");
        let target_jre_dir = runtimes_dir.join(format!("java-{}", major_version));

        fs::create_dir_all(&target_jre_dir).map_err(|e| e.to_string())?;

        let tar_path = runtimes_dir.join(format!("jre-{}.tar.gz", major_version));
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        fs::write(&tar_path, &bytes).map_err(|e| e.to_string())?;

        // Descomprimir usando el tar nativo de Linux
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(&tar_path)
            .arg("-C")
            .arg(&target_jre_dir)
            .arg("--strip-components=1")
            .status()
            .map_err(|e| format!("Error ejecutando tar: {}", e))?;

        fs::remove_file(&tar_path).ok();

        if status.success() {
            let java_bin = target_jre_dir.join("bin").join("java");
            Ok(format!("Java {} instalado exitosamente en {}", major_version, java_bin.display()))
        } else {
            Err("Fallo al descomprimir el runtime de Java".to_string())
        }
    }
}

pub struct MicrosoftAuthService;

impl MicrosoftAuthService {
    // Client ID público oficial vigente para lanzadores comunitarios de Minecraft (MultiMC / Prism Launcher)
    const CLIENT_ID: &'static str = "c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb";
    const USER_AGENT: &'static str = "OmnicraftLauncherStudio/2.0 (correakarl; linux)";

    /// Inicia el flujo Device Code Flow para autenticar al usuario
    pub async fn start_device_code_flow() -> Result<MicrosoftDeviceCodeResponse, String> {
        let client = reqwest::Client::builder()
            .user_agent(Self::USER_AGENT)
            .build()
            .map_err(|e| e.to_string())?;

        let params = [
            ("client_id", Self::CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
            ("response_type", "device_code"),
        ];

        let resp = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_txt = resp.text().await.unwrap_or_default();
            return Err(format!("Error de Microsoft OAuth: {}", err_txt));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        
        Ok(MicrosoftDeviceCodeResponse {
            device_code: data["device_code"].as_str().unwrap_or("").to_string(),
            user_code: data["user_code"].as_str().unwrap_or("").to_string(),
            verification_uri: data["verification_uri"].as_str().unwrap_or("https://www.microsoft.com/link").to_string(),
            expires_in: data["expires_in"].as_u64().unwrap_or(900),
            interval: data["interval"].as_u64().unwrap_or(5),
            message: data["message"].as_str().unwrap_or("Inicia sesión en microsoft.com/link con el código provisto").to_string(),
        })
    }

    /// Sondea el endpoint de Microsoft hasta que el usuario ingrese el código y completa el canje con Xbox Live y Mojang
    pub async fn poll_device_token(device_code: &str) -> Result<MicrosoftAccountSession, String> {
        let client = reqwest::Client::builder()
            .user_agent(Self::USER_AGENT)
            .build()
            .map_err(|e| e.to_string())?;

        let params = [
            ("client_id", Self::CLIENT_ID),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ];

        let resp = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        
        if let Some(err) = data["error"].as_str() {
            if err == "authorization_pending" {
                return Err("pending".to_string());
            } else if err == "slow_down" {
                return Err("pending".to_string());
            } else {
                return Err(format!("Microsoft OAuth: {}", data["error_description"].as_str().unwrap_or(err)));
            }
        }

        let ms_access_token = data["access_token"].as_str()
            .ok_or_else(|| "No se recibió access_token de Microsoft".to_string())?;

        // 1. Intercambiar MS token por Xbox Live (XBL)
        let xbl_payload = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", ms_access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let xbl_resp = client.post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&xbl_payload)
            .send()
            .await
            .map_err(|e| format!("Error conectando con Xbox Live: {}", e))?;

        if !xbl_resp.status().is_success() {
            return Err(format!("Fallo de autenticación con Xbox Live (HTTP {})", xbl_resp.status()));
        }

        let xbl_data: serde_json::Value = xbl_resp.json().await.map_err(|e| e.to_string())?;
        let xbl_token = xbl_data["Token"].as_str().ok_or_else(|| "Token XBL ausente".to_string())?;
        let user_hash = xbl_data["DisplayClaims"]["xui"][0]["uhs"].as_str()
            .ok_or_else(|| "User hash UHS ausente en Xbox Live".to_string())?;

        // 2. Obtener Ticket XSTS para Minecraft Services
        let xsts_payload = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let xsts_resp = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&xsts_payload)
            .send()
            .await
            .map_err(|e| format!("Error conectando con XSTS: {}", e))?;

        if !xsts_resp.status().is_success() {
            let err_val: serde_json::Value = xsts_resp.json().await.unwrap_or_default();
            let xerr = err_val["XErr"].as_i64().unwrap_or(0);
            if xerr == 2148916238 {
                return Err("La cuenta pertenece a un menor de edad y debe ser vinculada a una familia de Microsoft.".to_string());
            } else if xerr == 2148916233 {
                return Err("La cuenta no tiene un perfil de Xbox creado. Entra en xbox.com e inicia sesión primero.".to_string());
            }
            return Err(format!("Fallo al obtener ticket XSTS de Xbox: {}", err_val));
        }

        let xsts_data: serde_json::Value = xsts_resp.json().await.map_err(|e| e.to_string())?;
        let xsts_token = xsts_data["Token"].as_str().ok_or_else(|| "Token XSTS ausente".to_string())?;

        // 3. Login en api.minecraftservices.com con Xbox Token
        let mc_login_payload = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token)
        });

        let mc_resp = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&mc_login_payload)
            .send()
            .await
            .map_err(|e| format!("Error conectando con Mojang / Minecraft Services: {}", e))?;

        if !mc_resp.status().is_success() {
            let err_body = mc_resp.text().await.unwrap_or_default();
            return Err(format!("Error en api.minecraftservices.com: {}", err_body));
        }

        let mc_data: serde_json::Value = mc_resp.json().await.map_err(|e| e.to_string())?;
        let mc_access_token = mc_data["access_token"].as_str()
            .ok_or_else(|| "No se recibió access_token de Minecraft".to_string())?;

        // 4. Descargar perfil oficial de Minecraft (Nombre de usuario y UUID)
        let prof_resp = client.get("https://api.minecraftservices.com/minecraft/profile")
            .bearer_auth(mc_access_token)
            .send()
            .await
            .map_err(|e| format!("Error obteniendo perfil de Minecraft: {}", e))?;

        if prof_resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err("Esta cuenta de Microsoft no posee una copia comprada de Minecraft Java Edition.".to_string());
        } else if !prof_resp.status().is_success() {
            return Err(format!("Error al obtener perfil oficial de Minecraft ({})", prof_resp.status()));
        }

        let profile: serde_json::Value = prof_resp.json().await.map_err(|e| e.to_string())?;
        let username = profile["name"].as_str().unwrap_or("PlayerMS").to_string();
        let raw_uuid = profile["id"].as_str().unwrap_or("").to_string();

        let skin_url = profile["skins"].as_array()
            .and_then(|skins| skins.iter().find(|s| s["state"].as_str() == Some("ACTIVE")))
            .and_then(|s| s["url"].as_str())
            .map(|s| s.to_string());

        Ok(MicrosoftAccountSession {
            username,
            uuid: raw_uuid,
            access_token: mc_access_token.to_string(),
            skin_url,
        })
    }
}
