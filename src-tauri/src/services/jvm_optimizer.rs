// Optimizador y generador inteligente de argumentos JVM adaptados por versión
// Compatible con Java 8, Java 17 y Java 21-25 para Minecraft 1.0 hasta 1.21.x+

pub struct JvmOptimizer;

impl JvmOptimizer {
    /// Determina la versión mayor objetivo de Java según la versión de Minecraft
    pub fn target_java_major(mc_version: &str) -> u32 {
        let v = mc_version.trim();
        if v.starts_with("1.21") || v.starts_with("1.20.5") || v.starts_with("1.20.6") {
            21 // Java 21+ requerido por Mojang a partir de 1.20.5
        } else if v.starts_with("1.18") || v.starts_with("1.19") || v.starts_with("1.20") {
            17 // Java 17 requerido desde 1.18 hasta 1.20.4
        } else if v.starts_with("1.17") {
            16 // Java 16/17 introducido en 1.17
        } else {
            8  // Java 8 para 1.16.5 y anteriores (1.7.10, 1.12.2, etc.)
        }
    }

    /// Genera la lista de argumentos JVM optimizada según el runtime de Java real y la versión de Minecraft
    pub fn build_jvm_arguments(
        mc_version: &str,
        java_major: u32,
        ram_min_mb: u32,
        ram_max_mb: u32,
        preset: Option<&str>,
        custom_args: &str,
    ) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        // 1. Memoria Heap (Xms / Xmx)
        args.push(format!("-Xms{}M", ram_min_mb));
        args.push(format!("-Xmx{}M", ram_max_mb));

        // 2. Mitigación de seguridad Log4Shell (CVE-2021-44228) para todas las versiones
        args.push("-Dlog4j2.formatMsgNoLookups=true".to_string());

        // 3. Flags específicas por versión de Java y Minecraft
        let target_java = Self::target_java_major(mc_version);
        let effective_java = if java_major > 0 { java_major } else { target_java };

        // Parámetros de compatibilidad para Java 8 y versiones históricas
        if effective_java <= 8 {
            // Java 8: No soporta ZGC ni módulos --add-opens
            // Para Minecraft 1.7.10 e inferiores asignar MaxPermSize si es Java 7/8
            if mc_version.starts_with("1.7") || mc_version.starts_with("1.6") || mc_version.starts_with("1.5") {
                args.push("-XX:PermSize=128m".to_string());
                args.push("-XX:MaxPermSize=256m".to_string());
            }
            args.push("-XX:+UseG1GC".to_string());
            args.push("-XX:MaxGCPauseMillis=50".to_string());
            args.push("-XX:+DisableExplicitGC".to_string());
        } else {
            // Java 9+ (Java 16, 17, 21, 25): Modularity & JNA / Netty workarounds
            // Apertura de módulos Java para Forge/Fabric/NeoForge en Java moderno
            args.push("--add-opens=java.base/java.lang=ALL-UNNAMED".to_string());
            args.push("--add-opens=java.base/java.lang.reflect=ALL-UNNAMED".to_string());
            args.push("--add-opens=java.base/java.text=ALL-UNNAMED".to_string());
            args.push("--add-opens=java.base/java.util=ALL-UNNAMED".to_string());
            args.push("--add-opens=java.base/java.nio=ALL-UNNAMED".to_string());
            args.push("--add-opens=java.base/sun.security.action=ALL-UNNAMED".to_string());
            args.push("-Dsun.misc.URLClassPath.disableJarChecking=true".to_string());

            // Preset de Recolector de Basura adaptado
            let chosen_preset = preset.unwrap_or("aikar");
            match chosen_preset {
                "generational_zgc" => {
                    if effective_java >= 21 {
                        args.push("-XX:+UseZGC".to_string());
                        args.push("-XX:+ZGenerational".to_string());
                        args.push("-XX:+AlwaysPreTouch".to_string());
                    } else if effective_java >= 17 {
                        // Fallback ZGC clásico en Java 17
                        args.push("-XX:+UseZGC".to_string());
                        args.push("-XX:+AlwaysPreTouch".to_string());
                    } else {
                        // Fallback G1GC en Java <= 8
                        args.push("-XX:+UseG1GC".to_string());
                    }
                }
                "zgc" => {
                    if effective_java >= 17 {
                        args.push("-XX:+UseZGC".to_string());
                        args.push("-XX:+AlwaysPreTouch".to_string());
                    } else {
                        args.push("-XX:+UseG1GC".to_string());
                    }
                }
                "shenandoah" => {
                    if effective_java >= 17 {
                        args.push("-XX:+UseShenandoahGC".to_string());
                        args.push("-XX:+AlwaysPreTouch".to_string());
                        args.push("-XX:+UnlockExperimentalVMOptions".to_string());
                        args.push("-XX:ShenandoahGCHeuristics=adaptive".to_string());
                    } else {
                        args.push("-XX:+UseG1GC".to_string());
                    }
                }
                "graalvm_throughput" => {
                    args.push("-XX:+UseG1GC".to_string());
                    args.push("-XX:+UnlockDiagnosticVMOptions".to_string());
                    args.push("-XX:+UseJVMCICompiler".to_string());
                    args.push("-Dgraal.CompilerConfiguration=enterprise".to_string());
                    args.push("-XX:+AlwaysPreTouch".to_string());
                }
                "low_ram" => {
                    args.push("-XX:+UseSerialGC".to_string());
                    args.push("-XX:MinHeapFreeRatio=5".to_string());
                    args.push("-XX:MaxHeapFreeRatio=10".to_string());
                }
                "standard" => {
                    // Sin flags de GC invasivas
                }
                _ /* "aikar" o default */ => {
                    args.push("-XX:+UseG1GC".to_string());
                    args.push("-XX:+ParallelRefProcEnabled".to_string());
                    args.push("-XX:MaxGCPauseMillis=200".to_string());
                    args.push("-XX:+UnlockExperimentalVMOptions".to_string());
                    args.push("-XX:+DisableExplicitGC".to_string());
                    args.push("-XX:+AlwaysPreTouch".to_string());
                    args.push("-XX:G1NewSizePercent=30".to_string());
                    args.push("-XX:G1MaxNewSizePercent=40".to_string());
                    args.push("-XX:G1ReservePercent=20".to_string());
                    args.push("-XX:G1HeapWastePercent=5".to_string());
                    args.push("-XX:G1MixedGCCountTarget=4".to_string());
                    args.push("-XX:InitiatingHeapOccupancyPercent=15".to_string());
                    args.push("-XX:G1MixedGCLiveThresholdPercent=90".to_string());
                    args.push("-XX:G1RSetUpdatingPauseTimePercent=5".to_string());
                    args.push("-XX:SurvivorRatio=32".to_string());
                    args.push("-XX:+PerfDisableSharedMem".to_string());
                    args.push("-XX:MaxTenuringThreshold=1".to_string());
                }
            }

            // En Java 21-25 habilitar optimizaciones modernas de cabeceras compactas si está soportado
            if effective_java >= 21 {
                args.push("-Djava.awt.headless=false".to_string());
            }
        }

        // 4. Incorporar flags personalizadas del usuario previa sanitización anti-inyección
        let dangerous_tokens = ["rm ", "rmdir", "wget", "curl", "bash", "sh", ";", "&&", "||", "`", "$("];
        for token in custom_args.split_whitespace() {
            let t = token.trim();
            if t.is_empty() {
                continue;
            }
            if dangerous_tokens.iter().any(|&d| t.contains(d)) {
                continue;
            }
            // Evitar duplicar -Xms y -Xmx si el usuario los escribió manualmente
            if t.starts_with("-Xms") || t.starts_with("-Xmx") {
                continue;
            }
            // Si el Java efectivo es <= 17, no permitir -XX:+ZGenerational que causaría fallo de arranque
            if effective_java < 21 && t.contains("ZGenerational") {
                continue;
            }
            args.push(t.to_string());
        }

        args
    }

    /// Genera la cadena completa unificada de argumentos para perfiles JSON (launcher_profiles.json / HMCL)
    pub fn build_jvm_args_string(
        mc_version: &str,
        java_major: u32,
        ram_min_mb: u32,
        ram_max_mb: u32,
        preset: Option<&str>,
        custom_args: &str,
    ) -> String {
        let args = Self::build_jvm_arguments(mc_version, java_major, ram_min_mb, ram_max_mb, preset, custom_args);
        args.join(" ")
    }
}
