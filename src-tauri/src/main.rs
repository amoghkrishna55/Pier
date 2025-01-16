#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use axum::{
    extract::ConnectInfo,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use dotenv::dotenv;
use ngrok::prelude::*;
use std::env;
use std::net::SocketAddr;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::runtime::Runtime;
use webbrowser;

async fn open_youtube() -> impl IntoResponse {
    if webbrowser::open("https://www.youtube.com").is_ok() {
        (StatusCode::OK, "Opened YouTube in the default browser")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open YouTube")
    }
}

fn main() {
    dotenv().ok();
    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let system_tray_menu = SystemTrayMenu::new().add_item(quit);

    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();

    // Start the ngrok server in the background
    rt.spawn(async {
        // build our application with a single route
        let app = Router::new()
            .route(
                "/",
                get(
                    |ConnectInfo(remote_addr): ConnectInfo<SocketAddr>| async move {
                        format!("Hello, {remote_addr:?}!\r\n")
                    },
                ),
            )
            .route("/youtube", get(open_youtube));

        let tun = ngrok::Session::builder()
            // Read the token from the NGROK_AUTHTOKEN environment variable
            .authtoken_from_env()
            // Connect the ngrok session
            .connect()
            .await
            .unwrap()
            // Start a tunnel with an HTTP edge
            .http_endpoint()
            .domain(env::var("NGROK_DOMAIN").unwrap())
            .listen()
            .await
            .unwrap();

        println!("Tunnel started on URL: {:?}", tun.url());

        // Run it with an ngrok tunnel instead!
        axum::Server::builder(tun)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
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
}
