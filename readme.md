# Ru Berry
Rust application for integrating Zigbee devices designed to run on a Raspberry Pi.
The application expects a Zigbee2MQTT setup with an MQTT broker (e.g. Mosquitto).

Initial goal is to persist and visualize temperature and humidity data.

r2d2 is used for a connection pool, which only contains a single connection to the SQLite database.
This way there is no need to worry about async issues with SQLite.

The program consists of two parts, which are run on separate threads with tokio:
1. A MQTT client that listens for messages from Zigbee2MQTT and persists them to a SQLite database.
2. A web server that serves the data from the SQLite database.

## Building for Raspberry Pi
Building the project on the Pi takes a significant amount of time, 
so it is recommended to cross-compile the project on a more powerful machine.

Here's an example using [cross](https://github.com/cross-rs/cross). Verified to work on Debian.
1. Install cross
cargo install cross --git https://github.com/cross-rs/cross2
2. Build the executable for the Raspberry Pi
cross build --target aarch64-unknown-linux-gnu --release
3. Transfer the binary to the Raspberry Pi: You can use scp to transfer the binary:  
scp target/aarch64-unknown-linux-gnu/release/ru-berry pi@raspberrypi:/path/to/destination
4. Run the binary on the Raspberry Pi:  
./ru-berry

## Running the application
The application can be run either with a nohup command, or as a service.

### Running with nohup
To keep the application running after closing the terminal, use nohup.
```bash
nohup ./ru-berry &
```
#### Stopping the process
##### Find the process
```bash
ps -ef | grep ru-berry
```

##### Kill the process
```bash
kill <PID>
```

### Running as a service
Here's my service file `/etc/systemd/system/ru-berry.service`

Working directory is set, because `config.json` is loaded from the current directory.

```ini
[Unit]
Description=Ru Berry - Rust application for MQTT and web server
After=network.target
 
[Service]
User=user
WorkingDirectory=/home/user/ru-berry
ExecStart=/home/user/ru-berry/ru-berry
Restart=always
 
[Install]
WantedBy=multi-user.target
```

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
- [chrono](https://github.com/chronotope/chrono)
