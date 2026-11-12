// Parser Inteligente de Crash Logs y Errores en Rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDiagnosis {
    pub is_crash: bool,
    pub title: String,
    pub reason: String,
    pub solution: String,
}

pub struct CrashParserService;

impl CrashParserService {
    pub fn parse(log_text: &str) -> CrashDiagnosis {
        let t = log_text.to_lowercase();

        if t.contains("java.lang.outofmemoryerror") || t.contains("out of memory") {
            return CrashDiagnosis {
                is_crash: true,
                title: "Memoria RAM Insuficiente".to_string(),
                reason: "Minecraft se quedó sin memoria RAM para cargar los chunks o mods.".to_string(),
                solution: "Aumenta la memoria RAM asignada en los Ajustes de la instancia a al menos 4096 MB.".to_string(),
            };
        }

        if t.contains("unsupportedclassversionerror") || t.contains("has been compiled by a more recent version of the java runtime") {
            return CrashDiagnosis {
                is_crash: true,
                title: "Incompatibilidad de Versión de Java".to_string(),
                reason: "La versión de Minecraft o mod requiere un Java más moderno (ej. Java 21 para MC 1.20.5+).".to_string(),
                solution: "Cambia el Java en la pestaña Ajustes por Java 21 o Java 25.".to_string(),
            };
        }

        if t.contains("fabric loader has found conflicting") || t.contains("incompatible mod set") {
            return CrashDiagnosis {
                is_crash: true,
                title: "Conflicto o Dependencia Faltante en Mods".to_string(),
                reason: "Fabric detectó mods incompatibles entre sí o te falta una librería obligatoria (ej. Fabric API).".to_string(),
                solution: "Descarga 'Fabric API' en la tienda o desactiva el mod reportado en el registro.".to_string(),
            };
        }

        if t.contains("modloadingexception") || t.contains("fml:forge") {
            return CrashDiagnosis {
                is_crash: true,
                title: "Fallo de Carga de Mod en Forge/NeoForge".to_string(),
                reason: "Un mod no es compatible con esta versión exacta de Forge o requiere dependencias previas.".to_string(),
                solution: "Verifica que los mods instalados coincidan con la versión del ModLoader en la tabla de mods.".to_string(),
            };
        }

        if t.contains("exception in thread \"main\"") || t.contains("crash-reports") {
            return CrashDiagnosis {
                is_crash: true,
                title: "Fallo General al Iniciar".to_string(),
                reason: "Se produjo una excepción no controlada durante el arranque del juego.".to_string(),
                solution: "Revisa los últimos registros de la consola para identificar el mod causante.".to_string(),
            };
        }

        CrashDiagnosis {
            is_crash: false,
            title: "Ejecución Normal".to_string(),
            reason: "No se identificaron excepciones críticas.".to_string(),
            solution: "Todo en orden.".to_string(),
        }
    }
}
