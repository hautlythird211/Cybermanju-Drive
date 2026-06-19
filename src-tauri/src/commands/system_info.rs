use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_arch: String,
    pub hostname: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub total_disk_gb: f64,
    pub used_disk_gb: f64,
    pub kernel_version: String,
    pub uptime_seconds: u64,
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();

    let total_disk = sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .map(|d| d.total_space())
        .sum::<u64>();
    let used_disk = sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .map(|d| d.total_space() - d.available_space())
        .sum::<u64>();

    let cpu = sys.cpus().first();

    Ok(SystemInfo {
        os_name: System::name().unwrap_or_else(|| "Unknown".into()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".into()),
        os_arch: std::env::consts::ARCH.to_string(),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".into()),
        cpu_brand: cpu
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        cpu_cores: sys.cpus().len(),
        cpu_threads: sys.cpus().len(),
        total_memory_mb: total_memory / 1024 / 1024,
        used_memory_mb: used_memory / 1024 / 1024,
        total_disk_gb: (total_disk as f64) / 1024.0 / 1024.0 / 1024.0,
        used_disk_gb: (used_disk as f64) / 1024.0 / 1024.0 / 1024.0,
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
        uptime_seconds: System::uptime(),
    })
}
