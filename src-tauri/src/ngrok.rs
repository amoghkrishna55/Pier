use axum::{extract::ConnectInfo, http::StatusCode, response::IntoResponse, routing::get, Router};
use ngrok::prelude::*;
use ngrok::prelude::*;
use std::env;
use std::net::SocketAddr;
use webbrowser;

pub async fn open_youtube() -> impl IntoResponse {
    if webbrowser::open("https://www.youtube.com").is_ok() {
        (StatusCode::OK, "Opened YouTube in the default browser")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open YouTube")
    }
}

pub async fn invoke_ngrok() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        .await?
        // Start a tunnel with an HTTP edge
        .http_endpoint()
        .domain(env::var("NGROK_DOMAIN")?)
        .listen()
        .await?;

    println!("Tunnel started on URL: {:?}", tun.url());

    axum::Server::builder(tun)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
