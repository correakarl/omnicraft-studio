use sysinfo::{System, Pid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceResourceUsage {
    pub pid: u32,
    pub running: bool,
    pub cpu_percent: f32,
    pub ram_used_mb: u64,
    pub total_system_ram_mb: u64,
    pub instance_id: Option<String>,
    pub instance_name: Option<String>,
    pub username: Option<String>,
}

pub struct SysMonitor;

impl SysMonitor {
    pub fn get_process_usage(
        pid_val: u32,
        instance_id: Option<String>,
        instance_name: Option<String>,
        username: Option<String>,
    ) -> InstanceResourceUsage {
        let mut sys = System::new_all();
        sys.refresh_all();
        let total_system_ram_mb = sys.total_memory() / 1024 / 1024;

        let pid = Pid::from_u32(pid_val);
        if let Some(p) = sys.process(pid) {
            InstanceResourceUsage {
                pid: pid_val,
                running: true,
                cpu_percent: p.cpu_usage(),
                ram_used_mb: p.memory() / 1024 / 1024,
                total_system_ram_mb,
                instance_id,
                instance_name,
                username,
            }
        } else {
            InstanceResourceUsage {
                pid: pid_val,
                running: false,
                cpu_percent: 0.0,
                ram_used_mb: 0,
                total_system_ram_mb,
                instance_id,
                instance_name,
                username,
            }
        }
    }
}
