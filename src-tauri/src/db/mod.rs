// Repositorio y acceso a datos SQLite para Omnicraft v2.0
// Comparte la misma base de datos en ~/.omnicraft_studio/omnicraft.db
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceModel {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub icon: String,
    pub instance_type: String,
    pub server_port: u16,
    pub mc_version: String,
    pub loader_type: String,
    pub executable_type: String,
    pub launcher_jar: Option<String>,
    pub custom_executable_path: Option<String>,
    pub java_selection_mode: String,
    pub custom_java_path: Option<String>,
    pub window_launch_behavior: String,
    pub ram_min_mb: u32,
    pub ram_max_mb: u32,
    pub jvm_preset: Option<String>,
    pub jvm_args: Option<String>,
    pub width: u32,
    pub height: u32,
    pub assigned_username: Option<String>,
    pub notes: Option<String>,
    pub last_played: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: String,
    pub username: String,
    pub account_type: String,
    pub uuid: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationModel {
    pub id: String,
    pub category: String,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub badge: Option<String>,
    pub web_url: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModItemModel {
    pub instance_id: String,
    pub filename: String,
    pub mod_id: Option<String>,
    pub name: String,
    pub version: Option<String>,
    pub loader: Option<String>,
    pub active: bool,
    pub size_bytes: u64,
}

pub struct DbManager {
    db_path: PathBuf,
}

impl DbManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(home);
        path.push(".omnicraft_studio");
        fs::create_dir_all(&path).ok();
        path.push("omnicraft.db");
        Self { db_path: path }
    }

    pub fn get_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(conn)
    }

    pub fn get_all_instances(&self) -> Result<Vec<InstanceModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, folder, icon, COALESCE(instance_type, 'client'), COALESCE(server_port, 25565),
                    mc_version, loader_type, executable_type, launcher_jar, custom_executable_path,
                    COALESCE(java_selection_mode, 'auto'), custom_java_path, COALESCE(window_launch_behavior, 'minimize'),
                    COALESCE(ram_min_mb, 1024), COALESCE(ram_max_mb, 4096), jvm_preset, jvm_args,
                    COALESCE(width, 1280), COALESCE(height, 720), assigned_username, notes, COALESCE(last_played, 'Nunca')
             FROM instances ORDER BY rowid DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(InstanceModel {
                id: row.get(0)?,
                name: row.get(1)?,
                folder: row.get(2)?,
                icon: row.get(3)?,
                instance_type: row.get(4)?,
                server_port: row.get(5)?,
                mc_version: row.get(6)?,
                loader_type: row.get(7)?,
                executable_type: row.get(8)?,
                launcher_jar: row.get(9)?,
                custom_executable_path: row.get(10)?,
                java_selection_mode: row.get(11)?,
                custom_java_path: row.get(12)?,
                window_launch_behavior: row.get(13)?,
                ram_min_mb: row.get(14)?,
                ram_max_mb: row.get(15)?,
                jvm_preset: row.get(16)?,
                jvm_args: row.get(17)?,
                width: row.get(18)?,
                height: row.get(19)?,
                assigned_username: row.get(20)?,
                notes: row.get(21)?,
                last_played: row.get(22)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(inst) = r {
                list.push(inst);
            }
        }
        Ok(list)
    }

    pub fn get_accounts(&self) -> Result<Vec<AccountModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id, username, account_type, uuid, active FROM accounts ORDER BY active DESC, rowid DESC")?;
        let rows = stmt.query_map([], |row| {
            let active_int: i32 = row.get(4)?;
            Ok(AccountModel {
                id: row.get(0)?,
                username: row.get(1)?,
                account_type: row.get(2)?,
                uuid: row.get(3)?,
                active: active_int != 0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(acc) = r {
                list.push(acc);
            }
        }
        Ok(list)
    }

    pub fn get_active_account_credentials(&self, username: &str) -> Option<(String, String)> {
        self.get_account_full_credentials(username).map(|(u, tok, _ref, _skin)| (u, tok))
    }

    /// Retorna (uuid, access_token, refresh_token, skin_url) de la cuenta solicitada
    pub fn get_account_full_credentials(&self, username: &str) -> Option<(String, String, Option<String>, Option<String>)> {
        let conn = self.get_connection().ok()?;
        let mut stmt = conn.prepare(
            "SELECT data_json, uuid FROM accounts WHERE (username = ?1 OR id = ?1) ORDER BY (data_json IS NOT NULL AND data_json != '') DESC, active DESC LIMIT 1"
        ).ok()?;
        let mut rows = stmt.query(params![username]).ok()?;
        if let Some(row) = rows.next().ok().flatten() {
            let data_json: Option<String> = row.get(0).ok().flatten();
            let db_uuid: Option<String> = row.get(1).ok().flatten();
            
            if let Some(json_str) = data_json {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    let enc_token = val["enc_token"].as_str()
                        .or_else(|| val["token"].as_str())
                        .unwrap_or("");
                    let mut resolved_token = String::new();
                    if !enc_token.is_empty() {
                        // Descifrar con helper python si está cifrado con Fernet o usar token directo si no está cifrado
                        if enc_token.starts_with("gAAAAA") {
                            let script = format!(
                                "import os, json, sys\n\
                                from cryptography.fernet import Fernet\n\
                                kf = os.path.expanduser('~/.omnicraft_studio/.auth_vault.key')\n\
                                if os.path.exists(kf):\n    \
                                    with open(kf, 'rb') as f: c = Fernet(f.read().strip())\n    \
                                    print(c.decrypt('{}'.encode()).decode(), end='')\n",
                                enc_token
                            );
                            if let Ok(out) = std::process::Command::new("python3").arg("-c").arg(script).output() {
                                if out.status.success() {
                                    resolved_token = String::from_utf8_lossy(&out.stdout).to_string();
                                }
                            }
                        } else {
                            resolved_token = enc_token.to_string();
                        }
                    }

                    let uuid_str = val["uuid"].as_str()
                        .or(db_uuid.as_deref())
                        .unwrap_or("00000000-0000-0000-0000-000000000000");

                    let refresh_token = val["refresh_token"].as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let skin_url = val["skin_url"].as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    return Some((uuid_str.to_string(), resolved_token, refresh_token, skin_url));
                }
            }
        }
        None
    }

    pub fn get_official_recommendations(&self, category: &str) -> Result<Vec<RecommendationModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, category, title, author, description, badge, web_url, icon_url, downloads, rank
             FROM official_recommendations WHERE category = ?1 ORDER BY rank ASC, downloads DESC"
        )?;

        let rows = stmt.query_map(params![category], |row| {
            Ok(RecommendationModel {
                id: row.get(0)?,
                category: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                badge: row.get(5)?,
                web_url: row.get(6)?,
                icon_url: row.get(7)?,
                downloads: row.get(8)?,
                rank: row.get(9)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(rec) = r {
                list.push(rec);
            }
        }
        Ok(list)
    }

    pub fn get_instance_mods(&self, instance_id: &str) -> Result<Vec<ModItemModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT instance_id, filename, mod_id, name, version, loader, active, size_bytes
             FROM instance_mods WHERE instance_id = ?1 ORDER BY active DESC, name ASC"
        )?;

        let rows = stmt.query_map(params![instance_id], |row| {
            let active_int: i32 = row.get(6)?;
            Ok(ModItemModel {
                instance_id: row.get(0)?,
                filename: row.get(1)?,
                mod_id: row.get(2)?,
                name: row.get(3)?,
                version: row.get(4)?,
                loader: row.get(5)?,
                active: active_int != 0,
                size_bytes: row.get(7)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(m) = r {
                list.push(m);
            }
        }
        Ok(list)
    }

    pub fn create_instance(
        &self,
        name: &str,
        mc_version: &str,
        loader_type: &str,
        icon: &str,
        executable_type: &str,
        launcher_jar: Option<&str>,
        custom_exec: Option<&str>,
        java_mode: &str,
        custom_java: Option<&str>,
        ram_min: u32,
        ram_max: u32,
        jvm_preset: Option<&str>,
        jvm_args: Option<&str>,
        instance_type: &str,
        server_port: u16,
        assigned_username: Option<&str>,
    ) -> Result<String> {
        let conn = self.get_connection()?;
        let id = name.trim().replace(' ', "-").to_lowercase();
        let folder = name.trim().to_string();

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut inst_dir = PathBuf::from(&home);
        inst_dir.push(".omnicraft_studio");
        inst_dir.push("instances");
        inst_dir.push(&folder);
        let mut mc_dir = inst_dir.clone();
        mc_dir.push(".minecraft");
        fs::create_dir_all(&mc_dir).ok();
        
        let mut mods_dir = mc_dir.clone();
        mods_dir.push("mods");
        fs::create_dir_all(&mods_dir).ok();

        let mut config_dir = mc_dir.clone();
        config_dir.push("config");
        fs::create_dir_all(&config_dir).ok();

        conn.execute(
            "INSERT OR REPLACE INTO instances (
                id, name, folder, icon, mc_version, loader_type, executable_type,
                launcher_jar, custom_executable_path, java_selection_mode, custom_java_path,
                ram_min_mb, ram_max_mb, jvm_preset, jvm_args, instance_type, server_port, assigned_username
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                id, name, folder, icon, mc_version, loader_type, executable_type,
                launcher_jar, custom_exec, java_mode, custom_java,
                ram_min, ram_max, jvm_preset, jvm_args, instance_type, server_port, assigned_username
            ],
        )?;
        Ok(id)
    }

    pub fn update_instance(
        &self,
        id: &str,
        name: &str,
        icon: &str,
        mc_version: &str,
        loader_type: &str,
        executable_type: &str,
        launcher_jar: Option<&str>,
        custom_exec: Option<&str>,
        java_mode: &str,
        custom_java: Option<&str>,
        ram_min: u32,
        ram_max: u32,
        jvm_preset: Option<&str>,
        jvm_args: Option<&str>,
        width: u32,
        height: u32,
        assigned_username: Option<&str>,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE instances SET
                name = ?1, icon = ?2, mc_version = ?3, loader_type = ?4, executable_type = ?5,
                launcher_jar = ?6, custom_executable_path = ?7, java_selection_mode = ?8,
                custom_java_path = ?9, ram_min_mb = ?10, ram_max_mb = ?11, jvm_preset = ?12,
                jvm_args = ?13, width = ?14, height = ?15, assigned_username = ?16
            WHERE id = ?17",
            params![
                name, icon, mc_version, loader_type, executable_type,
                launcher_jar, custom_exec, java_mode, custom_java,
                ram_min, ram_max, jvm_preset, jvm_args, width, height, assigned_username, id
            ],
        )?;
        Ok(())
    }

    pub fn delete_instance(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        let mut folder: Option<String> = None;
        if let Ok(f) = conn.query_row("SELECT folder FROM instances WHERE id = ?1", params![id], |row| row.get(0)) {
            folder = Some(f);
        }

        conn.execute("DELETE FROM instance_mods WHERE instance_id = ?1", params![id])?;
        conn.execute("DELETE FROM play_sessions WHERE instance_id = ?1", params![id])?;
        conn.execute("DELETE FROM instances WHERE id = ?1", params![id])?;

        if let Some(folder_name) = folder {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let mut inst_dir = PathBuf::from(home);
            inst_dir.push(".omnicraft_studio");
            inst_dir.push("instances");
            inst_dir.push(folder_name);
            fs::remove_dir_all(inst_dir).ok();
        }
        Ok(())
    }

    pub fn set_active_account(&self, username: &str, account_type: &str, uuid: Option<&str>) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("UPDATE accounts SET active = 0", [])?;

        let updated = conn.execute(
            "UPDATE accounts SET active = 1, updated_at = CURRENT_TIMESTAMP WHERE username = ?1 OR id = ?1",
            params![username],
        )?;

        if updated == 0 {
            conn.execute(
                "INSERT INTO accounts (id, username, account_type, uuid, active)
                 VALUES (?1, ?2, ?3, ?4, 1)
                 ON CONFLICT(id) DO UPDATE SET active = 1, updated_at = CURRENT_TIMESTAMP",
                params![username, username, account_type, uuid],
            )?;
        }
        Ok(())
    }

    pub fn save_microsoft_account(
        &self,
        username: &str,
        uuid: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        skin_url: Option<&str>,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("UPDATE accounts SET active = 0", [])?;

        let user_id = format!("msa-{}", uuid);
        // Preservar refresh_token previo si el nuevo es None
        let existing_refresh = self.get_account_full_credentials(username)
            .and_then(|(_, _, r, _)| r);
        let final_refresh = refresh_token.or(existing_refresh.as_deref()).unwrap_or("");

        let data_json = serde_json::json!({
            "id": user_id,
            "username": username,
            "type": "microsoft",
            "uuid": uuid,
            "token": access_token,
            "enc_token": access_token,
            "refresh_token": final_refresh,
            "skin_url": skin_url.unwrap_or("")
        }).to_string();

        // Eliminar posibles duplicados sin token para el mismo nombre de usuario
        conn.execute(
            "DELETE FROM accounts WHERE username = ?1 AND id != ?2",
            params![username, user_id],
        ).ok();

        conn.execute(
            "INSERT INTO accounts (id, username, account_type, uuid, active, data_json, updated_at)
             VALUES (?1, ?2, 'microsoft', ?3, 1, ?4, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET username = ?2, account_type = 'microsoft', uuid = ?3, active = 1, data_json = ?4, updated_at = CURRENT_TIMESTAMP",
            params![user_id, username, uuid, data_json],
        )?;

        Ok(())
    }

    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        // Si no queda ninguna activa y hay cuentas, activar la primera disponible
        let mut stmt = conn.prepare("SELECT id FROM accounts WHERE active = 1")?;
        let has_active = stmt.exists([])?;
        if !has_active {
            conn.execute("UPDATE accounts SET active = 1 WHERE rowid = (SELECT MIN(rowid) FROM accounts)", [])?;
        }
        Ok(())
    }

    pub fn logout_and_clear_user_data(&self, username: &str) -> Result<()> {
        let conn = self.get_connection()?;
        // Eliminar la cuenta
        conn.execute("DELETE FROM accounts WHERE username = ?1 OR id = ?1", params![username])?;
        // Limpiar skins asociadas creadas o vinculadas
        conn.execute("DELETE FROM local_skins WHERE name LIKE ?1", params![format!("%{}%", username)])?;
        // Activar otra cuenta si existe
        let mut stmt = conn.prepare("SELECT id FROM accounts WHERE active = 1")?;
        let has_active = stmt.exists([])?;
        if !has_active {
            conn.execute("UPDATE accounts SET active = 1 WHERE rowid = (SELECT MIN(rowid) FROM accounts)", [])?;
        }

        // Limpiar posibles skins inyectadas en .minecraft/CustomSkinLoader/LocalSkin/skins/{username}.png de todas las instancias
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let instances_dir = PathBuf::from(home)
            .join(".omnicraft_studio")
            .join("instances");

        if let Ok(entries) = fs::read_dir(instances_dir) {
            for entry in entries.flatten() {
                let skin_file = entry.path()
                    .join(".minecraft")
                    .join("CustomSkinLoader")
                    .join("LocalSkin")
                    .join("skins")
                    .join(format!("{}.png", username));
                if skin_file.exists() {
                    fs::remove_file(skin_file).ok();
                }
            }
        }

        Ok(())
    }

    pub fn register_mod(
        &self,
        instance_id: &str,
        filename: &str,
        mod_id: Option<&str>,
        name: &str,
        version: Option<&str>,
        loader: Option<&str>,
        size_bytes: u64,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO instance_mods (instance_id, filename, mod_id, name, version, loader, active, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)
             ON CONFLICT(instance_id, filename) DO UPDATE SET
                name = excluded.name, version = excluded.version, active = 1, size_bytes = excluded.size_bytes",
            params![instance_id, filename, mod_id, name, version, loader, size_bytes],
        )?;
        Ok(())
    }

    pub fn unregister_mod(&self, instance_id: &str, filename: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "DELETE FROM instance_mods WHERE instance_id = ?1 AND filename = ?2",
            params![instance_id, filename],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT value FROM system_settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO system_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = CURRENT_TIMESTAMP",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_cached_versions(&self) -> Result<Vec<(String, String)>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT version_id, version_type FROM mojang_versions_cache ORDER BY rowid ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut list = Vec::new();
        for r in rows.flatten() {
            list.push(r);
        }
        Ok(list)
    }

    pub fn save_cached_versions(&self, versions: &[(String, String, Option<String>)]) -> Result<()> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM mojang_versions_cache", [])?;
        {
            let mut stmt = tx.prepare("INSERT INTO mojang_versions_cache (version_id, version_type, release_time) VALUES (?1, ?2, ?3)")?;
            for (vid, vtype, rtime) in versions {
                stmt.execute(params![vid, vtype, rtime])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkinModel {
    pub id: String,
    pub name: String,
    pub model_type: String, // "classic" o "slim"
    pub png_data_base64: String,
    pub texture_hash: Option<String>,
}

impl DbManager {
    pub fn get_all_local_skins(&self) -> Result<Vec<LocalSkinModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id, name, model_type, png_data_base64, texture_hash FROM local_skins ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(LocalSkinModel {
                id: row.get(0)?,
                name: row.get(1)?,
                model_type: row.get(2)?,
                png_data_base64: row.get(3)?,
                texture_hash: row.get(4)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows.flatten() {
            list.push(r);
        }
        Ok(list)
    }

    pub fn save_local_skin(&self, id: &str, name: &str, model_type: &str, base64_png: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO local_skins (id, name, model_type, png_data_base64, updated_at)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET name = ?2, model_type = ?3, png_data_base64 = ?4, updated_at = CURRENT_TIMESTAMP",
            params![id, name, model_type, base64_png],
        )?;
        Ok(())
    }

    pub fn delete_local_skin(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM local_skins WHERE id = ?1", params![id])?;
        Ok(())
    }
}
