//! Serves the web UI for all panels of a panel-config.toml and forwards each panel's
//! API, so the browser only ever talks to this server.

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use am03127_client::config::{Config, Panel};
use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, Method, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{any, get},
};
use clap::Parser;
use serde::Serialize;
use tower_http::services::ServeDir;

/// Normal API calls are quick; a firmware upload has to be flashed before the panel answers.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const OTA_TIMEOUT: Duration = Duration::from_secs(300);
/// Room for a firmware image (the OTA slot is 1.9 MB).
const MAX_BODY: usize = 4 * 1024 * 1024;

#[derive(Parser)]
#[command(about = "Web UI and API proxy for all AM03127 panels")]
struct Args {
    /// Path to the panel config file
    #[arg(long, default_value = "panel-config.toml")]
    config: PathBuf,
    /// Address to listen on
    #[arg(long, default_value = "127.0.0.1:8080")]
    listen: SocketAddr,
    /// Directory with the built web UI (am03127-webui/dist); API only if omitted
    #[arg(long)]
    web_root: Option<PathBuf>,
}

struct AppState {
    panels: Vec<Panel>,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct PanelInfo<'a> {
    name: &'a str,
    hidden: bool,
}

async fn list_panels(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let panels: Vec<PanelInfo> = state
        .panels
        .iter()
        .map(|p| PanelInfo {
            name: &p.name,
            hidden: p.hidden,
        })
        .collect();
    Json(panels).into_response()
}

/// Forwards `/api/panel/{name}/{path}` to `http://{address}/{path}` with method, body and
/// content type, and passes the panel's status and body back.
async fn proxy(
    State(state): State<Arc<AppState>>,
    Path((name, path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(panel) = state.panels.iter().find(|p| p.name == name) else {
        return (StatusCode::NOT_FOUND, format!("Unknown panel '{name}'")).into_response();
    };

    let timeout = if path == "ota" { OTA_TIMEOUT } else { REQUEST_TIMEOUT };
    let mut request = state
        .http
        .request(method, format!("http://{}/{path}", panel.address))
        .timeout(timeout)
        .body(body);
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        request = request.header(CONTENT_TYPE, content_type);
    }

    let result = async {
        let response = request.send().await?;
        let status = response.status();
        let content_type = response.headers().get(CONTENT_TYPE).cloned();
        Ok::<_, reqwest::Error>((status, content_type, response.bytes().await?))
    }
    .await;

    match result {
        Ok((status, content_type, body)) => {
            let mut response = (status, body).into_response();
            if let Some(content_type) = content_type {
                response.headers_mut().insert(CONTENT_TYPE, content_type);
            }
            response
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            format!("Panel '{name}' unreachable: {err}"),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    let state = Arc::new(AppState {
        panels: config.panels,
        http: reqwest::Client::new(),
    });

    let mut app = Router::new()
        .route("/api/panels", get(list_panels))
        .route("/api/panel/{name}/{*path}", any(proxy))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .with_state(state);
    if let Some(web_root) = &args.web_root {
        app = app.fallback_service(ServeDir::new(web_root));
    }

    let listener = tokio::net::TcpListener::bind(args.listen)
        .await
        .with_context(|| format!("failed to listen on {}", args.listen))?;
    println!("Listening on http://{}", args.listen);
    axum::serve(listener, app).await?;
    Ok(())
}
