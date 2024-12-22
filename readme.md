# Ru Berry
Rust application for integrating Zigbee devices designed to run on a Raspberry Pi.

Initial goal is to persist and visualize temperature and humidity data.

r2d2 is used for a connection pool, which only contains a single connection to the SQLite database.
This way there is no need to worry about async issues with SQLite.

## Stack
- [Rust](https://www.rust-lang.org/)
- [Mosquitto](https://mosquitto.org/) 
- [Zigbee2MQTT](https://www.zigbee2mqtt.io/).
- [RUMQTT](https://github.com/bytebeamio/rumqtt/tree/main)
- [Rusqlite](https://github.com/rusqlite/rusqlite)
  - `features = ["bundled"]` handles installing SQLite
- [SQLite](https://www.sqlite.org/index.html)
- [Serde](https://serde.rs/)
- [r2d2](https://github.com/sfackler/r2d2)
