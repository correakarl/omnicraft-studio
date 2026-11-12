// Omnicraft Launcher Studio v2.0 - Core Engine en Rust
// Autor: correakarl

pub mod db;
pub mod services;
pub mod engine;

pub use db::{DbManager, InstanceModel, AccountModel, RecommendationModel, ModItemModel, LocalSkinModel};
pub use services::java_service::{JavaService, JavaRuntime};
pub use services::download_manager::{DownloadManager, DownloadItem};
pub use services::profile_sync::ProfileSyncService;
pub use services::modrinth_service::{ModrinthClient, ModSearchResult};
pub use services::crash_parser::{CrashParserService, CrashDiagnosis};
pub use services::server_builder::ServerBuilderService;
pub use services::modrinth_download::ModrinthDownloadService;
pub use services::curseforge_service::{CurseForgeService, UnifiedSearchResult};
pub use services::mojang_manifest::{MojangManifestService, GameVersionEntry};
pub use services::skin_bridge::SkinBridgeService;
pub use services::content_tools::{ContentToolsService, WorldSaveInfo, LogOutput};
pub use services::content_manager::{ContentManagerService, ContentItem};
pub use services::malware_scanner::{MalwareScannerService, ScanResult};
pub use services::update_checker::{UpdateCheckerService, AppUpdateInfo};
pub use services::loader_installer::LoaderInstallerService;
pub use services::jre_provisioner::{JreProvisionerService, MicrosoftAuthService, MicrosoftDeviceCodeResponse};
pub use services::modpack_installer::{ModpackInstallerService, ModpackInstallProgress};
pub use services::compatibility_checker::{CompatibilityCheckerService, CompatibilityReport, CompatibilityIssue, ModMetadata};
pub use services::jvm_optimizer::JvmOptimizer;
pub use services::shared_storage::SharedStorageService;
pub use services::sys_monitor::{SysMonitor, InstanceResourceUsage};
pub use engine::{LaunchEngine, ProcessInfo};

pub fn get_core_status() -> &'static str {
    "Motor nativo sincronizado y listo"
}
