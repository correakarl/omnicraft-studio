// Servicio de descarga y aprovisionamiento automático de Java OpenJDK (Adoptium / Temurin)
// y autenticación nativa Device Code Flow de Microsoft
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
pub struct MicrosoftAuthResult {
    pub success: bool,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub error_message: Option<String>,
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
    const CLIENT_ID: &'static str = "00000000402b5328"; // ID oficial de Azure pública para Minecraft Launcher

    /// Inicia el flujo Device Code Flow para autenticar al usuario
    pub async fn start_device_code_flow() -> Result<MicrosoftDeviceCodeResponse, String> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", Self::CLIENT_ID),
            ("scope", "service::user.auth.xboxlive.com::MBI_SSL"),
            ("response_type", "device_code"),
        ];

        let resp = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Error de Microsoft OAuth: {}", resp.status()));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        
        Ok(MicrosoftDeviceCodeResponse {
            device_code: data["device_code"].as_str().unwrap_or("").to_string(),
            user_code: data["user_code"].as_str().unwrap_or("").to_string(),
            verification_uri: data["verification_uri"].as_str().unwrap_or("https://microsoft.com/link").to_string(),
            expires_in: data["expires_in"].as_u64().unwrap_or(900),
            interval: data["interval"].as_u64().unwrap_or(5),
            message: data["message"].as_str().unwrap_or("Inicia sesión en microsoft.com/link con el código provisto").to_string(),
        })
    }

    /// Sondea el endpoint de Microsoft hasta que el usuario ingrese el código o expire
    pub async fn poll_device_token(device_code: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
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
            } else {
                return Err(err.to_string());
            }
        }

        let access_token = data["access_token"].as_str().ok_or_else(|| "No se recibió access_token".to_string())?;
        Ok(access_token.to_string())
    }
}
