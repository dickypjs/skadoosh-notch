use std::process::Command;

pub struct PowerManager;

impl PowerManager {
    pub fn turn_off_screen() {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, HWND_BROADCAST, SC_MONITORPOWER, WM_SYSCOMMAND};
            let _ = SendMessageW(
                HWND(HWND_BROADCAST.0 as _),
                WM_SYSCOMMAND,
                WPARAM(SC_MONITORPOWER as _),
                LPARAM(2), // 2 = OFF
            );
        }
    }

    pub fn lock_pc() {
        let _ = Command::new("rundll32.exe")
            .args(["user32.dll,LockWorkStation"])
            .spawn();
    }

    pub fn sleep_pc() {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Power::SetSuspendState;
            use windows::Win32::Foundation::BOOLEAN;
            let _ = SetSuspendState(BOOLEAN(0), BOOLEAN(1), BOOLEAN(1));
        }
    }

    pub fn set_screen_timeout(minutes: u32) {
        let _ = Command::new("powercfg.exe")
            .args(["/change", "monitor-timeout-ac", &minutes.to_string()])
            .spawn();
        let _ = Command::new("powercfg.exe")
            .args(["/change", "monitor-timeout-dc", &minutes.to_string()])
            .spawn();
    }

    pub fn set_sleep_timeout(minutes: u32) {
        let _ = Command::new("powercfg.exe")
            .args(["/change", "standby-timeout-ac", &minutes.to_string()])
            .spawn();
        let _ = Command::new("powercfg.exe")
            .args(["/change", "standby-timeout-dc", &minutes.to_string()])
            .spawn();
    }
}
