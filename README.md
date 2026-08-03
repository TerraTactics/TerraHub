# TerraHub

Open-source agricultural edge gateway from [TerraTactics](https://github.com/TerraTactics).

TerraHub runs on Debian-based SBCs (Raspberry Pi, Radxa, Orange Pi, and similar) and bridges a **TerraLink** field mesh to the **FarmPilot** cloud SaaS.

## Role

- Manage TerraLink routing via a LoRa radio coprocessor
- Buffer telemetry offline (SQLite) and sync when connectivity returns
- Run local automation when the internet is unavailable
- Provide a local web/admin UI for gateway setup and FarmPilot pairing
- Discover and onboard pre-built TerraLink nodes (claiming happens in FarmPilot)

## Related projects

- [TerraLink](https://github.com/TerraTactics/TerraLink) — mesh protocol and node firmware (Apache-2.0)
- FarmPilot — commercial cloud SaaS (separate product)

## License

Licensed under the [Apache License 2.0](LICENSE).
