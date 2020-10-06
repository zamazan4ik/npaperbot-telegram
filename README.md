# npaperbot-telegram
Search C++ proposals in Telegram.

### Dependencies
* Rust 1.44 or higher
* Cargo

### How to build
* Clone this repository
* `cargo build --release`

### How to run
I recommend to run this bot as a service(e.g. as systemd service) on a machine.
Also Docker images are available here: https://hub.docker.com/repository/docker/zamazan4ik/npaperbot-telegram

### How to use
Currently the only way to use the bot is inline mode. Write a C++ proposal number in any paired brackets (e.g. `[p1000]` or `{p1000}`) and the bot will return all correspoding results.

Search functionality isn't supported yet (WIP). Stay tuned :)

