# Ru Berry
Rust application for integrating Zigbee devices designed to run on a Raspberry Pi.

Initial goal is to persist and visualize temperature and humidity data.

## Stack
- [Rust](https://www.rust-lang.org/)
- [Mosquitto](https://mosquitto.org/) 
- [Zigbee2MQTT](https://www.zigbee2mqtt.io/).
- [RUMQTT](https://github.com/bytebeamio/rumqtt/tree/main)
- [Rusqlite](https://github.com/rusqlite/rusqlite)
  - `features = ["bundled"]` handles installing SQLite
- [SQLite](https://www.sqlite.org/index.html)
- [Serde](https://serde.rs/)
