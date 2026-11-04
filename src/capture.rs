use std::process::Command;

pub struct CaptureManager;

impl CaptureManager {
    // Tangkap full screen via tool bawaan Windows / Snipping tool
    pub fn snip_area() {
        // Snipping tool area mode
        let _ = Command::new("explorer.exe")
            .arg("ms-screenclip:")
            .spawn();
    }

    pub fn capture_full() {
        // PrintScreen trigger
        let _ = Command::new("powershell.exe")
            .args(["-Command", "[System.Windows.Forms.SendKeys]::SendWait('{PRTSC}')"])
            .spawn();
    }

    pub fn record_screen() {
        // Windows Game Bar capture screen record
        let _ = Command::new("explorer.exe")
            .arg("ms-xbox-gamebar:")
            .spawn();
    }
}
