mod audio;
mod bridge;
mod protocol;
mod runtime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    runtime::run()
}
