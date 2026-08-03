# TerraHub

Open-source agricultural edge gateway from [TerraTactics](https://github.com/TerraTactics).

TerraHub runs on Debian-based SBCs (Raspberry Pi, Radxa, Orange Pi, and similar) and bridges a **TerraLink** field mesh to the **TerraTactics** cloud.

## Role

- Manage TerraLink routing via a LoRa radio coprocessor (UART / USB-serial)
- Buffer telemetry offline (SQLite) and sync when connectivity returns
- Run local automation when the internet is unavailable
- Provide a local web/admin UI for gateway setup and TerraTactics cloud pairing
- Discover and onboard pre-built TerraLink nodes (claiming happens in the TerraTactics cloud)

## Architecture

```
TerraHub (Linux / Debian)
├── radio/          RadioTransport — stub, UART loopback, or real serial
├── stack/          TerraLink RX dispatch + claim → Configuration 0x07
├── registry/       Discovered / claimed device table
├── buffer/         SQLite offline telemetry queue
├── cloud/          TerraTactics cloud MQTT agent placeholder + claim stub
└── admin/          Local HTTP setup wizard (axum)
         │
         │ UART / USB-serial (length-prefixed TerraLink frames)
         ▼
   LoRa coprocessor (ESP32 + radio, etc.)
```

Protocol details live in the [TerraLink PROTOCOL.md](https://github.com/TerraTactics/TerraLink/blob/main/PROTOCOL.md).

## Build

Requires Rust 1.74+ and a sibling checkout of [TerraLink](https://github.com/TerraTactics/TerraLink) at `../TerraLink` (path dependency).

```bash
# from TerraHub/
cargo build --release
cargo test
cargo run -- --config config/terrahub.example.toml
```

### Admin preview

With the daemon running:

- UI: <http://127.0.0.1:8080/>
- Status JSON: <http://127.0.0.1:8080/api/status>
- Claim stub (after a node has been discovered):

```bash
curl -s -X POST http://127.0.0.1:8080/api/devices/claim \
  -H "content-type: application/json" \
  -d '{"identity":"TL-000127","routing_addr":66}'
```

Keep the `cargo run` process running (or install the systemd unit) so the preview stays up.

The local setup pages reuse TerraTactics visual tokens (olive / ivory / gold) and load **Jost** + **DM Sans** from Google Fonts when the hub has outbound HTTPS; without network they fall back to `system-ui` / `Segoe UI`.

### Radio backends

| `radio.backend` | Purpose |
|-----------------|---------|
| `stub` | Default in-memory transport (no serial) |
| `loopback` | Same UART framing as hardware, in-memory (tests) |
| `uart` / `usb-serial` | Real serial to the LoRa coprocessor |

Example UART config:

```toml
[radio]
backend = "uart"
device = "/dev/ttyUSB0"   # Linux; on Windows use e.g. "COM3"
baud = 115200
```

On-wire UART framing: little-endian `u16` length, then a complete TerraLink frame (header + payload + CRC-16/MODBUS).

### Discovery → claim → config

1. Node sends Discovery (`0x04`) → hub registry marks **pending**
2. TerraTactics cloud (or local `POST /api/devices/claim`) issues a claim with a routing address
3. Hub sends Configuration (`0x07`) over the radio and marks the device **claimed**

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
- [TerraCloud](https://github.com/TerraTactics/TerraCloud) — Community Edition cloud portal (basics in progress; see its `docs/ROADMAP.md`)
- [TerraTactics](https://terratactics.com.au) — product site

FarmPilot.io under FarmPilotIO is queued **after** Terra basics — stand-alone rebrand with extra features; greenfield cloud only (do not touch live production).

## License

Licensed under the [Apache License 2.0](LICENSE).
