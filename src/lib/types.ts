export interface InstanceModel {
  id: string;
  name: string;
  folder: string;
  icon: string;
  instance_type: 'client' | 'server';
  server_port: number;
  mc_version: string;
  loader_type: string;
  executable_type: string;
  launcher_jar?: string | null;
  custom_executable_path?: string | null;
  java_selection_mode: string;
  custom_java_path?: string | null;
  window_launch_behavior: string;
  ram_min_mb: number;
  ram_max_mb: number;
  jvm_preset?: string | null;
  jvm_args?: string | null;
  width: number;
  height: number;
  assigned_username?: string | null;
  notes?: string | null;
  last_played?: string | null;
}

export interface AccountModel {
  id: string;
  username: string;
  account_type: 'offline' | 'microsoft';
  uuid?: string | null;
  active: boolean;
}

export interface JavaRuntime {
  path: string;
  version: number;
  is_64bit: boolean;
  vendor: string;
}

export interface RecommendationModel {
  id: string;
  category: string;
  title: string;
  author?: string | null;
  description?: string | null;
  badge?: string | null;
  web_url: string;
  icon_url?: string | null;
  downloads: number;
  rank: number;
}

export interface AppUpdateModel {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_notes: string;
  download_url: string;
}

export interface ScanResultModel {
  is_clean: boolean;
  infected_entries: string[];
  threat_level: 'safe' | 'warning' | 'critical';
  description: string;
}

export interface ModItemModel {
  instance_id: string;
  filename: string;
  mod_id?: string | null;
  name: string;
  version?: string | null;
  loader?: string | null;
  active: boolean;
  size_bytes: number;
}

export interface ModSearchResult {
  id: string;
  source: 'Modrinth' | 'CurseForge';
  slug: string;
  title: string;
  description: string;
  author: string;
  downloads: number;
  icon_url?: string | null;
  project_type: string;
  web_url?: string;
  loaders: string[];
  versions: string[];
}

export interface GameVersionEntry {
  id: string;
  version_type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
  release_time?: string | null;
}

export interface LocalSkinModel {
  id: string;
  name: string;
  model_type: 'classic' | 'slim';
  png_data_base64: string;
  texture_hash?: string | null;
}

export interface WorldSaveModel {
  folder_name: string;
  display_name: string;
  size_bytes: number;
  last_modified: string;
  icon_base64?: string | null;
  game_mode?: string | null;
  backups?: string[];
}

export interface InstanceLogOutput {
  lines: string[];
  total_lines: number;
  last_updated: string;
}

export interface ContentItemModel {
  name: string;
  filename: string;
  category: 'mods' | 'shaderpacks' | 'resourcepacks';
  enabled: boolean;
  size_bytes: number;
}

export interface MicrosoftDeviceCodeModel {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
  message: string;
}

export interface MicrosoftAccountSession {
  username: string;
  uuid: string;
  access_token: string;
  skin_url?: string | null;
}

export interface DownloadTaskModel {
  id: string;
  title: string;
  source: string;
  status: string;
  progress: number;
  completed: boolean;
}

export interface CompatibilityIssueModel {
  severity: 'error' | 'warning' | 'info';
  mod_filename: string;
  title: string;
  description: string;
  recommendation: string;
}

export interface CompatibilityReportModel {
  is_compatible: boolean;
  checked_mods_count: number;
  instance_mc_version: string;
  instance_loader: string;
  issues: CompatibilityIssueModel[];
  mods: Array<{
    filename: string;
    mod_id: string;
    name: string;
    version: string;
    detected_loader: string;
  }>;
}

export interface SharedCacheItemModel {
  filename: string;
  category: string;
  size_bytes: number;
  is_linked_to_instance: boolean;
}

export interface InstanceResourceUsage {
  pid: number;
  running: boolean;
  cpu_percent: number;
  ram_used_mb: number;
  total_system_ram_mb: number;
  instance_id?: string;
  instance_name?: string;
  username?: string;
}
