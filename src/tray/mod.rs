#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
#[allow(unused_imports)]
pub use linux::{send_notification, spawn_tray, TrayHandleWrapper};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
#[allow(unused_imports)]
pub use windows::{send_notification, spawn_tray, TrayHandleWrapper};
