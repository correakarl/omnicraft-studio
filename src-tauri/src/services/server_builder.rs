// Configurador automático de Servidores Dedicados y aceptación de EULA
use std::path::Path;
use std::fs;

pub struct ServerBuilderService;

impl ServerBuilderService {
    pub fn setup_server(server_dir: &Path, port: u16) -> Result<(), String> {
        fs::create_dir_all(server_dir).map_err(|e| e.to_string())?;

        // 1. Aceptar automáticamente EULA oficial (eula.txt)
        let eula_path = server_dir.join("eula.txt");
        let eula_content = format!(
            "# By changing the setting below to TRUE you are indicating your agreement to the EULA (https://aka.ms/MinecraftEULA).\n# Generated automatically by Omnicraft Launcher Studio v2.0\neula=true\n"
        );
        fs::write(eula_path, eula_content).map_err(|e| e.to_string())?;

        // 2. server.properties con puerto configurable
        let props_path = server_dir.join("server.properties");
        if !props_path.exists() {
            let props_content = format!(
                "server-port={}\nquery.port={}\nmotd=Omnicraft Studio Server\nonline-mode=false\nenable-command-block=true\nsync-chunk-writes=true\n",
                port, port
            );
            fs::write(props_path, props_content).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
