#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dotenv::dotenv;
use sysinfo;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

mod ngrok;
mod warp;

async fn handle_ngrok_and_warp() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Err(e) = ngrok::invoke_ngrok().await {
        eprintln!("Failed to invoke ngrok: {}", e);
        let invoke = warp::connect_to_warp();

        if invoke {
            loop {
                let status = warp::get_warp_status();

                if status.contains("Status update: Connected") {
                    ngrok::invoke_ngrok().await.unwrap();
                    break;
                } else if status.contains("Reason: No Network") {
                    println!("nO netwrok");
                    break;
                }
                println!("Waiting for WARP to connect...");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn get_address() -> String {
    "http://localhost:3000".to_string() // Replace with your actual address logic
}

fn main() {
    dotenv().ok();
    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let system_tray_menu = SystemTrayMenu::new().add_item(quit);

    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();

    rt.spawn(async move {
        if let Err(e) = handle_ngrok_and_warp().await {
            eprintln!("Error in handle_ngrok_and_warp: {}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            get_address,
            warp::get_warp_status,
            warp::disconnect_from_warp
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(system_tray_menu))
        .on_system_tray_event(|app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                SystemTrayEvent::LeftClick {
                    position: _,
                    size: _,
                    ..
                } => {
                    let window = app.get_window("main").unwrap();
                    // window
                    //     .set_size(tauri::Size::Logical(tauri::LogicalSize {
                    //         width: 300.0,
                    //         height: 1000.0,
                    //     }))
                    //     .unwrap();
                    let _ = window.move_window(Position::TrayCenter);

                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                SystemTrayEvent::RightClick {
                    position: _,
                    size: _,
                    ..
                } => {
                    println!("system tray received a right click");
                }
                SystemTrayEvent::DoubleClick {
                    position: _,
                    size: _,
                    ..
                } => {
                    println!("system tray received a double click");
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "hide" => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(is_focused) => {
                // detect click outside of the focused window and hide the app
                if !is_focused {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Keep the runtime alive until the application exits
    rt.block_on(async {
        // Your async code here if needed
    });
}
