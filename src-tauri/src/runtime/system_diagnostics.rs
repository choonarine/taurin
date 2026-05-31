use std::{
    env, fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn log_startup_environment() {
    let executable = env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|error| format!("<unavailable: {error}>"));
    let cwd = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|error| format!("<unavailable: {error}>"));
    let available_parallelism = std::thread::available_parallelism()
        .map(|count| count.get().to_string())
        .unwrap_or_else(|error| format!("<unavailable: {error}>"));
    let hardware = hardware_summary();

    println!(
        "[taurin:startup] time=\"{}\" os=\"{}\" os_detail=\"{}\" family={} arch={} pointer_width={} available_parallelism={} memory_total_gib={} cpu=\"{}\" gpu=\"{}\" executable=\"{}\" cwd=\"{}\"",
        startup_time(),
        env::consts::OS,
        os_detail(),
        env::consts::FAMILY,
        env::consts::ARCH,
        std::mem::size_of::<usize>() * 8,
        available_parallelism,
        hardware.memory_total_gib.unwrap_or_else(|| "<unavailable>".to_string()),
        hardware.cpu.unwrap_or_else(|| "<unavailable>".to_string()),
        hardware.gpu.unwrap_or_else(|| "<unavailable>".to_string()),
        executable,
        cwd,
    );
}

pub fn log_window_displays(window: &tauri::WebviewWindow) {
    match window.available_monitors() {
        Ok(monitors) => {
            let current_index = current_monitor_index(window, &monitors);
            let displays = monitors
                .iter()
                .enumerate()
                .map(|(index, monitor)| {
                    format!(
                        "#{}{}:{}:{}x{}@{},{}:work={}x{}+{},{}:scale={}",
                        index + 1,
                        if Some(index) == current_index {
                            "*"
                        } else {
                            ""
                        },
                        monitor.name().map(String::as_str).unwrap_or("<unknown>"),
                        monitor.size().width,
                        monitor.size().height,
                        monitor.position().x,
                        monitor.position().y,
                        monitor.work_area().size.width,
                        monitor.work_area().size.height,
                        monitor.work_area().position.x,
                        monitor.work_area().position.y,
                        monitor.scale_factor()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");

            println!(
                "[taurin:display] count={} game_display_index={} displays=\"{}\"",
                monitors.len(),
                current_index
                    .map(|index| (index + 1).to_string())
                    .unwrap_or_else(|| "<unavailable>".to_string()),
                displays
            );
        }
        Err(error) => println!("[taurin:display] monitors=<unavailable: {error}>"),
    }
}

fn current_monitor_index(
    window: &tauri::WebviewWindow,
    monitors: &[tauri::Monitor],
) -> Option<usize> {
    let current = window.current_monitor().ok().flatten()?;

    monitors.iter().position(|monitor| {
        monitor.name() == current.name()
            && monitor.position() == current.position()
            && monitor.size() == current.size()
            && (monitor.scale_factor() - current.scale_factor()).abs() < f64::EPSILON
    })
}

struct HardwareSummary {
    cpu: Option<String>,
    gpu: Option<String>,
    memory_total_gib: Option<String>,
}

fn hardware_summary() -> HardwareSummary {
    HardwareSummary {
        cpu: cpu_name(),
        gpu: gpu_name(),
        memory_total_gib: memory_total_gib(),
    }
}

fn startup_time() -> String {
    #[cfg(windows)]
    if let Some(value) = command_line(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            "Get-Date -Format 'yyyy-MM-dd HH:mm:ss zzz'",
        ],
    ) {
        return value;
    }

    #[cfg(unix)]
    if let Some(value) = command_line("date", &["+%Y-%m-%d %H:%M:%S %z"]) {
        return value;
    }

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix_seconds={}", duration.as_secs()),
        Err(error) => format!("<unavailable: {error}>"),
    }
}

fn os_detail() -> String {
    #[cfg(windows)]
    if let Some(value) = command_line("powershell", &["-NoProfile", "-Command", "(Get-CimInstance Win32_OperatingSystem).Caption + ' build ' + (Get-CimInstance Win32_OperatingSystem).BuildNumber"]) {
        return value;
    }

    #[cfg(target_os = "macos")]
    if let Some(value) = command_line("sw_vers", &["-productVersion"]) {
        return format!("macOS {value}");
    }

    #[cfg(target_os = "linux")]
    if let Some(value) = linux_os_release_pretty_name() {
        return value;
    }

    env::consts::OS.to_string()
}

fn cpu_name() -> Option<String> {
    #[cfg(windows)]
    {
        return command_line(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name",
            ],
        );
    }

    #[cfg(target_os = "macos")]
    {
        return command_line("sysctl", &["-n", "machdep.cpu.brand_string"]);
    }

    #[cfg(target_os = "linux")]
    {
        return linux_cpu_name();
    }

    #[allow(unreachable_code)]
    None
}

fn gpu_name() -> Option<String> {
    #[cfg(windows)]
    {
        return command_line(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_VideoController | ForEach-Object { $_.Name }",
            ],
        );
    }

    #[cfg(target_os = "macos")]
    {
        return command_line(
            "sh",
            &[
                "-c",
                "system_profiler SPDisplaysDataType 2>/dev/null | awk -F': ' '/Chipset Model|Vendor/ {print $2}' | paste -sd '; ' -",
            ],
        );
    }

    #[cfg(target_os = "linux")]
    {
        return command_line(
            "sh",
            &[
                "-c",
                "command -v lspci >/dev/null 2>&1 && lspci | grep -Ei 'vga|3d|display' | sed 's/^[^:]*: //' | paste -sd '; ' -",
            ],
        );
    }

    #[allow(unreachable_code)]
    None
}

fn memory_total_gib() -> Option<String> {
    #[cfg(windows)]
    {
        let memory_kb = command_line(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_OperatingSystem).TotalVisibleMemorySize",
            ],
        )?;
        return memory_kb
            .parse::<f64>()
            .ok()
            .map(|kb| format!("{:.2}", kb / 1024.0 / 1024.0));
    }

    #[cfg(target_os = "macos")]
    {
        let bytes = command_line("sysctl", &["-n", "hw.memsize"])?;
        return bytes
            .parse::<f64>()
            .ok()
            .map(|bytes| format!("{:.2}", bytes / 1024.0 / 1024.0 / 1024.0));
    }

    #[cfg(target_os = "linux")]
    {
        return linux_memory_total_gib();
    }

    #[allow(unreachable_code)]
    None
}

#[cfg(target_os = "linux")]
fn linux_os_release_pretty_name() -> Option<String> {
    fs::read_to_string("/etc/os-release")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches('"').to_string())
}

#[cfg(target_os = "linux")]
fn linux_cpu_name() -> Option<String> {
    fs::read_to_string("/proc/cpuinfo")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|line| {
            line.split_once(':')
                .map(|(_, value)| value.trim().to_string())
        })
}

#[cfg(target_os = "linux")]
fn linux_memory_total_gib() -> Option<String> {
    fs::read_to_string("/proc/meminfo")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|kb| kb.parse::<f64>().ok())
        .map(|kb| format!("{:.2}", kb / 1024.0 / 1024.0))
}

fn command_line(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("; ");

    (!value.is_empty()).then_some(value)
}
