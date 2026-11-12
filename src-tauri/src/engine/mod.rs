// Motor de Ejecución de Procesos Minecraft en Rust
// Sincroniza configuraciones, aísla HOME/APPDATA y ejecuta lanzadores embebidos o servidores
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::services::jvm_optimizer::JvmOptimizer;
use crate::services::java_service::JavaService;

pub mod direct_launcher;
use direct_launcher::{DirectLaunchConfig, DirectLaunchPlan};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub instance_id: String,
    pub instance_name: String,
    pub pid: u32,
    pub instance_type: String,
    pub username: String,
    pub is_msa: bool,
    pub running: bool,
}

pub struct ActiveProcess {
    pub child: Child,
    pub instance_name: String,
    pub username: String,
    pub is_msa: bool,
}

pub struct LaunchEngine {
    processes: Arc<Mutex<HashMap<String, ActiveProcess>>>,
}

impl LaunchEngine {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_running(&self, instance_id: &str) -> bool {
        let mut procs = self.processes.lock().unwrap();
        if let Some(entry) = procs.get_mut(instance_id) {
            match entry.child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(instance_id);
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn is_msa_in_use(&self) -> bool {
        let mut procs = self.processes.lock().unwrap();
        let mut dead = Vec::new();
        let mut msa_active = false;

        for (id, entry) in procs.iter_mut() {
            match entry.child.try_wait() {
                Ok(None) => {
                    if entry.is_msa {
                        msa_active = true;
                    }
                }
                _ => {
                    dead.push(id.clone());
                }
            }
        }

        for d in dead {
            procs.remove(&d);
        }

        msa_active
    }

    pub fn get_active_instances(&self) -> Vec<ProcessInfo> {
        let mut procs = self.processes.lock().unwrap();
        let mut list = Vec::new();
        let mut dead = Vec::new();

        for (id, entry) in procs.iter_mut() {
            match entry.child.try_wait() {
                Ok(None) => {
                    list.push(ProcessInfo {
                        instance_id: id.clone(),
                        instance_name: entry.instance_name.clone(),
                        pid: entry.child.id(),
                        instance_type: "client".to_string(),
                        username: entry.username.clone(),
                        is_msa: entry.is_msa,
                        running: true,
                    });
                }
                _ => {
                    dead.push(id.clone());
                }
            }
        }

        for d in dead {
            procs.remove(&d);
        }

        list
    }

    pub fn terminate(&self, instance_id: &str) -> bool {
        let mut procs = self.processes.lock().unwrap();
        if let Some(mut entry) = procs.remove(instance_id) {
            entry.child.kill().ok();
            true
        } else {
            false
        }
    }

    /// Envía un comando por stdin al proceso del servidor
    pub fn send_server_command(&self, instance_id: &str, command: &str) -> Result<(), String> {
        let mut procs = self.processes.lock().unwrap();
        if let Some(entry) = procs.get_mut(instance_id) {
            if let Some(ref mut stdin) = entry.child.stdin {
                let cmd_line = format!("{}\n", command.trim());
                stdin.write_all(cmd_line.as_bytes()).map_err(|e| format!("Error escribiendo a stdin: {}", e))?;
                stdin.flush().map_err(|e| e.to_string())?;
                Ok(())
            } else {
                Err("El proceso no tiene un canal stdin interactivo abierto.".to_string())
            }
        } else {
            Err("La instancia del servidor no se encuentra en ejecución.".to_string())
        }
    }

    pub fn launch_instance(
        &self,
        instance_id: &str,
        instance_name: &str,
        instance_type: &str,
        mc_version: &str,
        loader_type: &str,
        mc_dir: &Path,
        root_dir: &Path,
        java_bin: &str,
        ram_min_mb: u32,
        ram_max_mb: u32,
        jvm_preset: Option<&str>,
        jvm_args: Option<&str>,
        executable_type: &str,
        launcher_jar: Option<&str>,
        custom_exec_path: Option<&str>,
        _server_port: u16,
        username: &str,
        uuid: Option<&str>,
        access_token: Option<&str>,
        user_type: &str,
        width: u32,
        height: u32,
    ) -> Result<u32, String> {
        if self.is_running(instance_id) {
            return Err(format!("La instancia '{}' ya se encuentra en ejecución", instance_name));
        }

        let is_server = instance_type == "server";
        let mut cmd = Command::new(java_bin);

        // Variables de entorno aisladas
        cmd.env("HOME", root_dir);
        cmd.env("APPDATA", root_dir);
        cmd.env("XDG_DATA_HOME", root_dir.join(".local").join("share"));
        cmd.env("XDG_CONFIG_HOME", root_dir.join(".config"));

        if let Some(java_home) = Path::new(java_bin).parent().and_then(|p| p.parent()) {
            cmd.env("JAVA_HOME", java_home);
        }

        // Inspeccionar versión de Java en tiempo de ejecución o deducirla por MC
        let java_major = JavaService::inspect_java(Path::new(java_bin))
            .map(|j| j.version)
            .unwrap_or_else(|| JvmOptimizer::target_java_major(mc_version));

        // Construir argumentos JVM inteligentemente ajustados según la versión
        let version_jvm_args = JvmOptimizer::build_jvm_arguments(
            mc_version,
            java_major,
            ram_min_mb,
            ram_max_mb,
            jvm_preset,
            jvm_args.unwrap_or(""),
        );

        for arg in &version_jvm_args {
            cmd.arg(arg);
        }

        cmd.arg(format!("-Duser.home={}", root_dir.to_string_lossy()));
        cmd.arg(format!("-Dhmcl.dir={}", root_dir.join(".hmcl").to_string_lossy()));

        let exec_cwd = if is_server {
            let server_jar = mc_dir.join("server.jar");
            if !server_jar.exists() {
                return Err("server.jar no encontrado en la instancia del servidor.".to_string());
            }
            cmd.arg("-jar").arg(server_jar).arg("nogui");
            mc_dir.to_path_buf()
        } else {
            // Modo Cliente: Comprobar si se puede ejecutar directamente con el Motor Nativo Directo
            let is_direct_requested = executable_type == "direct" 
                || launcher_jar.map(|j| j.to_lowercase().contains("direct") || j.to_lowercase().contains("nativo")).unwrap_or(false);

            let versions_dir = mc_dir.join("versions");
            let can_use_direct = versions_dir.exists();

            if is_direct_requested || launcher_jar.is_none() {
                if !can_use_direct {
                    return Err(format!(
                        "La instancia '{}' está configurada en modo Motor Directo, pero aún no tiene archivos de versión instalados en '{}'. Instala un modloader o una versión primero.",
                        instance_name, versions_dir.display()
                    ));
                }

                let launch_config = DirectLaunchConfig {
                    instance_name,
                    mc_dir,
                    mc_version,
                    loader_type,
                    java_bin,
                    jvm_args: &version_jvm_args,
                    username,
                    uuid,
                    access_token,
                    user_type,
                    width,
                    height,
                };

                match DirectLaunchPlan::build(&launch_config) {
                    Ok(plan) => {
                        // Inyectar Classpath nativo y clase principal del cliente
                        cmd.arg("-cp").arg(&plan.classpath);
                        cmd.arg(&plan.main_class);
                        for g_arg in plan.game_args {
                            cmd.arg(g_arg);
                        }
                        mc_dir.to_path_buf()
                    }
                    Err(err) => {
                        return Err(format!("Error en el Motor Nativo Directo: {}", err));
                    }
                }
            } else {
                Self::fallback_to_jar_launcher(&mut cmd, mc_dir, root_dir, launcher_jar, custom_exec_path, executable_type)?
            }
        };

        cmd.current_dir(exec_cwd);
        if is_server {
            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        } else {
            cmd.stdin(Stdio::null());
            // Para clientes, redirigir stdout y stderr hacia latest.log en mc_dir/logs/
            let logs_dir = mc_dir.join("logs");
            fs::create_dir_all(&logs_dir).ok();
            let log_file_path = logs_dir.join("latest.log");
            if let Ok(file) = fs::File::create(&log_file_path) {
                if let Ok(file_err) = file.try_clone() {
                    cmd.stdout(Stdio::from(file));
                    cmd.stderr(Stdio::from(file_err));
                } else {
                    cmd.stdout(Stdio::null());
                    cmd.stderr(Stdio::null());
                }
            } else {
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());
            }
        }

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                let mut procs = self.processes.lock().unwrap();
                procs.insert(
                    instance_id.to_string(),
                    ActiveProcess {
                        child,
                        instance_name: instance_name.to_string(),
                        username: username.to_string(),
                        is_msa: user_type == "msa",
                    },
                );
                Ok(pid)
            }
            Err(e) => Err(format!("No se pudo iniciar el proceso de la instancia: {}", e)),
        }
    }

    fn fallback_to_jar_launcher(
        cmd: &mut Command,
        mc_dir: &Path,
        root_dir: &Path,
        launcher_jar: Option<&str>,
        custom_exec_path: Option<&str>,
        executable_type: &str,
    ) -> Result<PathBuf, String> {
        let mut jar_to_run: Option<PathBuf> = None;

        if executable_type == "custom_jar" {
            if let Some(p) = custom_exec_path {
                let pb = PathBuf::from(p);
                if pb.exists() {
                    jar_to_run = Some(pb);
                }
            }
        }

        if jar_to_run.is_none() {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let mut launchers_dir = PathBuf::from(home);
            launchers_dir.push("Games");
            launchers_dir.push("Omnicraft-v2");
            launchers_dir.push("lanzadores");

            let target_name = launcher_jar.unwrap_or("HMCL.jar");
            let mut selected_jar = launchers_dir.join(target_name);

            if !selected_jar.exists() {
                selected_jar = launchers_dir.join("HMCL.jar");
            }
            if selected_jar.exists() {
                jar_to_run = Some(selected_jar);
            }
        }

        let final_jar = jar_to_run.ok_or_else(|| "No se encontró ningún lanzador ejecutable (.jar) disponible.".to_string())?;

        cmd.arg("-jar")
            .arg(&final_jar)
            .arg("--workDir")
            .arg(mc_dir);

        Ok(root_dir.to_path_buf())
    }
}
