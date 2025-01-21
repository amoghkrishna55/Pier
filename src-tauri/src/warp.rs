use std::process::Command;

const WARP_PATH: &str = "/Applications/Cloudflare WARP.app/Contents/Resources/warp-cli";

pub fn connect_to_warp() -> bool {
    let output = Command::new(WARP_PATH)
        .arg("connect")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Connected to WARP successfully");
        true
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to connect to WARP: {}", stderr);
        false
    }
}

#[tauri::command]
pub fn disconnect_from_warp() -> bool {
    let output = Command::new(WARP_PATH)
        .arg("disconnect")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Disconnected from WARP successfully");
        true
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to disconnect from WARP: {}", stderr);
        false
    }
}

#[tauri::command]
pub fn get_warp_status() -> String {
    let output = Command::new(WARP_PATH)
        .arg("status")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        stdout.to_string()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        stderr.to_string()
    }
}
