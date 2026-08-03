# TerraHub

Open-source agricultural edge gateway from [TerraTactics](https://github.com/TerraTactics).

TerraHub runs on Debian-based SBCs (Raspberry Pi, Radxa, Orange Pi, and similar) and bridges a **TerraLink** field mesh to the **FarmPilot** cloud SaaS.

## Role

- Manage TerraLink routing via a LoRa radio coprocessor
- Buffer telemetry offline (SQLite) and sync when connectivity returns
- Run local automation when the internet is unavailable
- Provide a local web/admin UI for gateway setup and FarmPilot pairing
- Discover and onboard pre-built TerraLink nodes (claiming happens in FarmPilot)

## Architecture (skeleton)

```
TerraHub (Linux / Debian)
├── radio/          RadioTransport trait — stub today; UART/USB-serial later
├── stack/          TerraLink RX dispatch (uses `terralink` crate)
├── registry/       Discovered / claimed device table
├── buffer/         SQLite offline telemetry queue
├── cloud/          FarmPilot MQTT agent placeholder
└── admin/          Local HTTP setup wizard stubs (axum)
         │
         │ UART / USB-serial (future)
         ▼
   LoRa coprocessor (ESP32 + radio, etc.)
```

Protocol details live in the [TerraLink PROTOCOL.md](https://github.com/TerraTactics/TerraLink/blob/main/PROTOCOL.md).

## Build

Requires Rust 1.74+ and a sibling checkout of [TerraLink](https://github.com/TerraTactics/TerraLink) at `../TerraLink` (path dependency).

```bash
# from TerraHub/
cargo build --release
cargo run -- --config config/terrahub.example.toml
```

Admin UI (setup only): <http://127.0.0.1:8080/> — status JSON at `/api/status`.

### Debian / Raspberry Pi

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/TerraTactics/TerraLink.git
git clone https://github.com/TerraTactics/TerraHub.git
cd TerraHub
cargo build --release

sudo install -m 755 target/release/terrahub /usr/local/bin/terrahub
sudo mkdir -p /etc/terrahub /var/lib/terrahub
sudo cp config/terrahub.example.toml /etc/terrahub/config.toml
sudo cp systemd/terrahub.service /etc/systemd/system/
# create user, adjust serial group, then:
# sudo systemctl enable --now terrahub
```

## Related projects

- [TerraLink](https://github.com/TerraTactics/TerraLink) — mesh protocol and node firmware (Apache-2.0)
- FarmPilot — commercial cloud SaaS (separate product)

## License

Licensed under the [Apache License 2.0](LICENSE).
