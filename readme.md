# Ru Berry
Rust application for integrating Zigbee devices designed to run on a Raspberry Pi.

Initial goal is to persist and visualize temperature and humidity data.

r2d2 is used for a connection pool, which only contains a single connection to the SQLite database.
This way there is no need to worry about async issues with SQLite.

The program consists of two parts, which are run on separate threads with tokio:
1. A MQTT client that listens for messages from Zigbee2MQTT and persists them to a SQLite database.
2. A web server that serves the data from the SQLite database.

## Stack and dependencies
- [Rust](https://www.rust-lang.org/)
- [Mosquitto](https://mosquitto.org/) 
- [Zigbee2MQTT](https://www.zigbee2mqtt.io/).
- [RUMQTT](https://github.com/bytebeamio/rumqtt/tree/main)
- [Rusqlite](https://github.com/rusqlite/rusqlite)
  - `features = ["bundled"]` handles installing SQLite
- [SQLite](https://www.sqlite.org/index.html)
- [Serde](https://serde.rs/)
- [r2d2](https://github.com/sfackler/r2d2)
- [tokio](https://tokio.rs/)
- [warp](https://github.com/seanmonstar/warp)
