[![Rust](https://github.com/JuliDi/mbfse/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/JuliDi/mbfse/actions/workflows/rust.yml)

# mbfse

Most basic* file share ever.

*Less than 100 lines of code

## Prerequisites
* A working fileserver, e.~g. Caddy or nginx that serves files copied to a specific directory

## Usage

1. Set environment variables to point _mbfse_ to the fileserver:
    * `BASE_URL`: Base-URL of your files server, e.g. `BASE_URL=https://filesserver.example.com`
    * `STORAGE_PATH`: Directory that is served by the file server, e.g. `STORAGE_PATH=/var/www/fileserver`
   
   **Example**
   
   Uploaded file gets stored to `/var/www/fileserver/xyz.jpg`.

   Accessing `https://fileserver.example.com/xyz.jpg` will download the file.

2. Optional: Set the port _mbfse_ will be listening on by setting the environment variable `ROCKET_PORT=<PORT>`
3. Start _mbfse_ with `$ cargo run`

## Systemd service example
Build _mbfse_ wtih `$ cargo build --release` and copy the binary from `target/release` to `/usr/local/bin`.

Add the following systemd service to automatically start _mbfse_:

```systemd
[Unit]
Description=mbfse
Documentation=https://github.com/JuliDi/mbfse
After=sys-subsystem-net-devices-eth0.device
StartLimitIntervalSec=500
StartLimitBurst=5

[Service]
User=www-data
Group=www-data
ExecStart=/usr/local/bin/mbfse
TimeoutStopSec=5s
LimitNOFILE=1048576
LimitNPROC=512
PrivateTmp=true
ProtectSystem=full
AmbientCapabilities=CAP_NET_BIND_SERVICE
Environment=ROCKET_PORT=<ENTER ROCKET PORT>
Environment=ROCKET_ADDRESS="127.0.0.1"
Restart=always
RestartSec=5s
Environment=BASE_URL=<BASE URL OF THE FILESERVER>
Environment=STORAGE_PATH=<PATH SERVED BY FILESERVER>

[Install]
WantedBy=multi-user.target
```