//! Local HTTP admin — gateway setup / pairing placeholders only.
//!
//! Visual tokens follow the public TerraTactics site
//! (<https://terratactics.com.au>): olive/ivory/gold palette, Jost + DM Sans.

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
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TerraHub Setup — TerraTactics</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=DM+Sans:opsz,wght@9..40,400;9..40,500;9..40,700&family=Jost:wght@400;500;600&display=swap" rel="stylesheet">
  <style>
    :root {
      --tt-ink: #17251D;
      --tt-olive: #344126;
      --tt-olive-700: #2A351E;
      --tt-olive-900: #1C2415;
      --tt-olive-300: #6B7A55;
      --tt-lime: #DDF34A;
      --tt-lime-ink: #2A351E;
      --tt-gold: #D9A521;
      --tt-gold-600: #BC8B15;
      --tt-gold-100: #F7EDD2;
      --tt-soil: #8A583A;
      --tt-sage: #DCE7D7;
      --tt-ivory: #F6F3EA;
      --tt-ivory-deep: #EFEADC;
      --tt-paper: #FFFFFF;
      --tt-line: #E1DACA;
      --tt-line-strong: #CFC6B1;
      --tt-body: #5F5A50;
      --tt-muted: #8B857A;
      --tt-good: #4E7C3F;
      --tt-good-bg: #E7F0E1;
      --tt-watch: #B8811C;
      --tt-watch-bg: #F8EFD8;
      --tt-alert: #A6482C;
      --tt-alert-bg: #F6E3DB;
      --tt-info: #4F769D;
      --tt-info-bg: #E3EBF3;
      --tt-font-display: "Jost", "Century Gothic", system-ui, sans-serif;
      --tt-font-body: "DM Sans", system-ui, -apple-system, "Segoe UI", sans-serif;
      --tt-step--2: .75rem;
      --tt-step--1: .8125rem;
      --tt-step-0: 1.0625rem;
      --tt-step-1: 1.25rem;
      --tt-step-2: clamp(1.375rem, 1.2rem + .7vw, 1.75rem);
      --tt-step-3: clamp(1.625rem, 1.35rem + 1.1vw, 2.25rem);
      --tt-tracking-eyebrow: .14em;
      --tt-tracking-button: .12em;
      --tt-gutter: clamp(1.25rem, 4vw, 2.5rem);
      --tt-space-1: .5rem;
      --tt-space-2: .875rem;
      --tt-space-3: 1.25rem;
      --tt-space-4: 2rem;
      --tt-space-5: 3rem;
      --tt-radius: 10px;
      --tt-shadow: 0 14px 40px -18px rgba(23, 37, 29, .28);
      --tt-ease: cubic-bezier(.22, .61, .36, 1);
    }

    *, *::before, *::after { box-sizing: border-box; }
    html { -webkit-text-size-adjust: 100%; }
    body {
      margin: 0;
      min-height: 100vh;
      font-family: var(--tt-font-body);
      font-size: var(--tt-step-0);
      line-height: 1.55;
      color: var(--tt-ink);
      background:
        radial-gradient(ellipse 80% 50% at 10% -10%, rgba(221, 243, 74, .12), transparent 55%),
        radial-gradient(ellipse 60% 40% at 100% 0%, rgba(217, 165, 33, .08), transparent 50%),
        var(--tt-ivory);
    }
    a { color: var(--tt-olive); text-decoration: none; transition: color .22s var(--tt-ease); }
    a:hover { color: var(--tt-gold-600); }

    .site-header {
      background: var(--tt-olive);
      color: #fff;
    }
    .site-header__inner {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--tt-space-3);
      max-width: 840px;
      margin-inline: auto;
      padding: 1.15rem var(--tt-gutter);
      min-height: 4.5rem;
    }
    .brand {
      display: inline-flex;
      align-items: center;
      gap: .75rem;
      color: #fff;
    }
    .brand:hover { color: #fff; }
    .brand__mark {
      display: grid;
      place-items: center;
      width: 2.5rem;
      height: 2.5rem;
      border-radius: 50%;
      background: var(--tt-lime);
      color: var(--tt-olive-900);
      font-family: var(--tt-font-display);
      font-size: 1rem;
      font-weight: 600;
      letter-spacing: -.02em;
      flex: none;
    }
    .brand__word {
      font-family: var(--tt-font-display);
      font-size: 1.15rem;
      font-weight: 400;
      letter-spacing: .14em;
      line-height: 1.15;
      text-transform: uppercase;
    }
    .brand__word strong { font-weight: 600; }
    .brand__word small {
      display: block;
      margin-top: .3rem;
      font-size: .5625rem;
      font-weight: 500;
      letter-spacing: .16em;
      color: #ffffffa8;
    }
    .header-meta {
      font-family: var(--tt-font-display);
      font-size: var(--tt-step--2);
      font-weight: 500;
      letter-spacing: .1em;
      text-transform: uppercase;
      color: #ffffffa8;
      text-align: right;
    }

    main {
      max-width: 840px;
      margin-inline: auto;
      padding: var(--tt-space-5) var(--tt-gutter) var(--tt-space-6);
    }

    .tt-eyebrow {
      display: flex;
      align-items: center;
      gap: .75rem;
      margin: 0 0 var(--tt-space-2);
      font-family: var(--tt-font-display);
      font-size: var(--tt-step--1);
      font-weight: 600;
      letter-spacing: var(--tt-tracking-eyebrow);
      text-transform: uppercase;
      color: var(--tt-gold-600);
    }
    .tt-eyebrow::before {
      content: "";
      width: 2.25rem;
      height: 2px;
      background: currentColor;
      flex: none;
    }
    h1 {
      margin: 0 0 var(--tt-space-2);
      font-family: var(--tt-font-display);
      font-size: var(--tt-step-3);
      font-weight: 500;
      line-height: 1.15;
      letter-spacing: -.01em;
      color: var(--tt-ink);
    }
    .lead {
      margin: 0 0 var(--tt-space-4);
      font-size: var(--tt-step-1);
      line-height: 1.6;
      color: var(--tt-body);
      max-width: 36rem;
    }

    .panel {
      background: var(--tt-paper);
      border: 1px solid var(--tt-line);
      border-radius: var(--tt-radius);
      box-shadow: var(--tt-shadow);
      padding: var(--tt-space-4);
      margin-bottom: var(--tt-space-4);
    }
    .panel h2 {
      margin: 0 0 var(--tt-space-3);
      font-family: var(--tt-font-display);
      font-size: var(--tt-step-2);
      font-weight: 500;
      color: var(--tt-ink);
    }
    .panel p {
      margin: 0 0 var(--tt-space-3);
      color: var(--tt-body);
    }
    .panel p:last-child { margin-bottom: 0; }

    .status-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(9.5rem, 1fr));
      gap: var(--tt-space-2);
      margin-bottom: var(--tt-space-3);
    }
    .stat {
      background: var(--tt-ivory);
      border: 1px solid var(--tt-line);
      padding: var(--tt-space-2) var(--tt-space-3);
    }
    .stat__label {
      display: block;
      font-family: var(--tt-font-display);
      font-size: var(--tt-step--2);
      font-weight: 600;
      letter-spacing: var(--tt-tracking-eyebrow);
      text-transform: uppercase;
      color: var(--tt-muted);
      margin-bottom: .35rem;
    }
    .stat__value {
      font-family: var(--tt-font-display);
      font-size: var(--tt-step-1);
      font-weight: 500;
      color: var(--tt-ink);
      word-break: break-all;
    }

    .steps {
      list-style: none;
      margin: 0;
      padding: 0;
      counter-reset: setup;
    }
    .steps li {
      counter-increment: setup;
      display: flex;
      gap: var(--tt-space-3);
      align-items: flex-start;
      padding: var(--tt-space-3) 0;
      border-top: 1px solid var(--tt-line);
      color: var(--tt-body);
    }
    .steps li:first-child { border-top: 0; padding-top: 0; }
    .steps li::before {
      content: counter(setup, decimal-leading-zero);
      font-family: var(--tt-font-display);
      font-size: var(--tt-step--1);
      font-weight: 600;
      letter-spacing: .08em;
      color: var(--tt-gold-600);
      flex: none;
      min-width: 1.75rem;
      padding-top: .15rem;
    }

    .actions {
      display: flex;
      flex-wrap: wrap;
      gap: var(--tt-space-2);
      margin-top: var(--tt-space-3);
    }
    .tt-btn {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: .6rem;
      padding: .85rem 1.75rem;
      border: 2px solid var(--tt-gold);
      border-radius: 0;
      background: var(--tt-gold);
      color: #fff;
      font-family: var(--tt-font-display);
      font-size: var(--tt-step--1);
      font-weight: 600;
      letter-spacing: var(--tt-tracking-button);
      text-transform: uppercase;
      line-height: 1;
      cursor: pointer;
      transition: background-color .24s var(--tt-ease), border-color .24s var(--tt-ease), color .24s var(--tt-ease), transform .24s var(--tt-ease);
    }
    .tt-btn:hover {
      background: var(--tt-gold-600);
      border-color: var(--tt-gold-600);
      color: #fff;
      transform: translateY(-1px);
    }
    .tt-btn--ghost {
      background: transparent;
      border-color: var(--tt-olive);
      color: var(--tt-olive);
    }
    .tt-btn--ghost:hover {
      background: var(--tt-olive);
      border-color: var(--tt-olive);
      color: #fff;
    }

    .notice {
      background: var(--tt-watch-bg);
      border: 1px solid #e8d7a8;
      padding: var(--tt-space-3);
      color: var(--tt-olive-700);
      font-size: var(--tt-step--1);
    }
    .notice strong {
      font-family: var(--tt-font-display);
      font-weight: 600;
      letter-spacing: .06em;
      text-transform: uppercase;
      display: block;
      margin-bottom: .35rem;
      color: var(--tt-watch);
    }

    .device-list {
      list-style: none;
      margin: 0;
      padding: 0;
    }
    .device-list li {
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      gap: .5rem;
      padding: .75rem 0;
      border-top: 1px solid var(--tt-line);
      font-size: var(--tt-step--1);
    }
    .device-list li:first-child { border-top: 0; }
    .badge {
      font-family: var(--tt-font-display);
      font-size: .6875rem;
      font-weight: 600;
      letter-spacing: .1em;
      text-transform: uppercase;
      padding: .3rem .55rem;
      background: var(--tt-good-bg);
      color: var(--tt-good);
    }
    .badge--pending {
      background: var(--tt-watch-bg);
      color: var(--tt-watch);
    }
    .empty {
      color: var(--tt-muted);
      font-size: var(--tt-step--1);
      margin: 0;
    }

    .site-footer {
      max-width: 840px;
      margin-inline: auto;
      padding: 0 var(--tt-gutter) var(--tt-space-5);
      font-size: var(--tt-step--2);
      color: var(--tt-muted);
      letter-spacing: .04em;
    }
    .site-footer a { color: var(--tt-olive-300); }

    @media (max-width: 520px) {
      .site-header__inner { min-height: 4rem; }
      .header-meta { display: none; }
      .actions { flex-direction: column; }
      .tt-btn { width: 100%; }
    }
  </style>
</head>
<body>
  <header class="site-header">
    <div class="site-header__inner">
      <a class="brand" href="/">
        <span class="brand__mark" aria-hidden="true">TH</span>
        <span class="brand__word">Terra<strong>Hub</strong><small>TerraTactics gateway</small></span>
      </a>
      <p class="header-meta">Local setup</p>
    </div>
  </header>

  <main>
    <p class="tt-eyebrow">Gateway setup</p>
    <h1>Bring this hub onto your farm mesh</h1>
    <p class="lead">
      Configure radio, identity, and FarmPilot pairing here.
      Day-to-day device claiming stays in FarmPilot — not this local wizard.
    </p>

    <section class="panel" aria-labelledby="status-heading">
      <h2 id="status-heading">Hub status</h2>
      <div class="status-grid">
        <div class="stat">
          <span class="stat__label">Identity</span>
          <span class="stat__value" id="hub-identity">…</span>
        </div>
        <div class="stat">
          <span class="stat__label">Version</span>
          <span class="stat__value" id="hub-version">…</span>
        </div>
        <div class="stat">
          <span class="stat__label">Devices</span>
          <span class="stat__value" id="hub-devices">…</span>
        </div>
      </div>
      <ul class="device-list" id="device-list" hidden></ul>
      <p class="empty" id="device-empty">No TerraLink nodes discovered yet.</p>
      <div class="actions">
        <button type="button" class="tt-btn" id="refresh-status">Refresh status</button>
        <a class="tt-btn tt-btn--ghost" href="/api/status">Raw JSON</a>
      </div>
    </section>

    <section class="panel" aria-labelledby="setup-heading">
      <h2 id="setup-heading">Setup steps</h2>
      <ol class="steps" id="setup-steps">
        <li>Loading…</li>
      </ol>
      <div class="actions">
        <a class="tt-btn tt-btn--ghost" href="/api/setup">Setup API</a>
      </div>
    </section>

    <aside class="notice">
      <strong>FarmPilot is separate</strong>
      Claiming pre-built TerraLink nodes and day-to-day farm ops happen in the FarmPilot cloud product.
      This page is local LAN setup only.
    </aside>
  </main>

  <footer class="site-footer">
    TerraHub open-source gateway ·
    <a href="https://terratactics.com.au" rel="noopener">terratactics.com.au</a>
  </footer>

  <script>
    async function loadStatus() {
      try {
        const res = await fetch("/api/status");
        const data = await res.json();
        document.getElementById("hub-identity").textContent = data.hub_identity || "—";
        document.getElementById("hub-version").textContent = data.version || "—";
        const devices = Array.isArray(data.devices) ? data.devices : [];
        document.getElementById("hub-devices").textContent = String(devices.length);
        const list = document.getElementById("device-list");
        const empty = document.getElementById("device-empty");
        list.innerHTML = "";
        if (devices.length === 0) {
          list.hidden = true;
          empty.hidden = false;
        } else {
          empty.hidden = true;
          list.hidden = false;
          for (const d of devices) {
            const li = document.createElement("li");
            const claim = d.claim === "claimed" ? "claimed" : "pending";
            li.innerHTML =
              "<span><strong>" + escapeHtml(d.identity) + "</strong> · addr " +
              escapeHtml(String(d.routing_addr)) + "</span>" +
              '<span class="badge' + (claim === "pending" ? " badge--pending" : "") + '">' +
              claim + "</span>";
            list.appendChild(li);
          }
        }
      } catch (err) {
        document.getElementById("hub-identity").textContent = "unavailable";
        document.getElementById("hub-version").textContent = "—";
        document.getElementById("hub-devices").textContent = "—";
      }
    }

    async function loadSetup() {
      try {
        const res = await fetch("/api/setup");
        const data = await res.json();
        const ol = document.getElementById("setup-steps");
        ol.innerHTML = "";
        for (const step of data.steps || []) {
          const li = document.createElement("li");
          li.textContent = step;
          ol.appendChild(li);
        }
      } catch (err) {
        document.getElementById("setup-steps").innerHTML =
          "<li>Could not load setup steps.</li>";
      }
    }

    function escapeHtml(s) {
      return String(s)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
    }

    document.getElementById("refresh-status").addEventListener("click", loadStatus);
    loadStatus();
    loadSetup();
  </script>
</body>
</html>"#;

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
