//! Local HTTP admin — gateway setup / pairing placeholders only.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;

use crate::registry::{ClaimState, DeviceRegistry};

#[derive(Clone)]
struct AppState {
    hub_identity: String,
    registry: Arc<RwLock<DeviceRegistry>>,
}

#[derive(Serialize)]
struct StatusResponse {
    hub_identity: String,
    service: &'static str,
    version: &'static str,
    devices: Vec<DeviceDto>,
}

#[derive(Serialize)]
struct DeviceDto {
    identity: String,
    routing_addr: u16,
    claim: &'static str,
}

pub async fn serve(
    bind: &str,
    registry: Arc<RwLock<DeviceRegistry>>,
    hub_identity: String,
) -> anyhow::Result<()> {
    let state = AppState {
        hub_identity,
        registry,
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/api/status", get(status))
        .route("/api/setup", get(setup_placeholder))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>TerraHub Setup</title></head>
<body>
  <h1>TerraHub</h1>
  <p>Local gateway setup wizard (skeleton).</p>
  <ul>
    <li><a href="/api/status">JSON status</a></li>
    <li><a href="/api/setup">Setup steps (placeholder)</a></li>
  </ul>
  <p>Day-to-day device claiming happens in FarmPilot, not here.</p>
</body>
</html>"#,
    )
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let reg = state.registry.read().await;
    let devices = reg
        .list()
        .into_iter()
        .map(|d| DeviceDto {
            identity: d.identity,
            routing_addr: d.routing_addr,
            claim: match d.claim {
                ClaimState::Pending => "pending",
                ClaimState::Claimed => "claimed",
            },
        })
        .collect();
    Json(StatusResponse {
        hub_identity: state.hub_identity.clone(),
        service: "terrahub",
        version: env!("CARGO_PKG_VERSION"),
        devices,
    })
}

async fn setup_placeholder() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "steps": [
            "Set hub identity / hostname",
            "Configure radio serial device",
            "Pair with FarmPilot (token exchange TBD)",
            "Confirm TerraLink mesh hearability"
        ],
        "note": "Farm device claim UX is in FarmPilot, not this admin UI"
    }))
}
