# npaperbot-telegram
Search C++ proposals in Telegram.

### Dependencies
* [Rust](https://www.rust-lang.org/) 1.44 or newer
* Cargo

Older Rust compiler versions possibly able to build the project but I didn't test it.

### How to build
* Clone this repository
* `cargo build --release`

### How to run
I recommend to run this bot as a service(e.g. as systemd service) on a machine.
Also Docker images are available here: https://hub.docker.com/repository/docker/zamazan4ik/npaperbot-telegram

### Configuration
The bot can be configured only with environment variables. For now there are we support the following variables:

| Name | Description | Values | Default value | Required |
|------|-------------|--------|---------------|----------|
| TELOXIDE_TOKEN | Telegram bot token | Any valid and registered Telegram bot token | None | All mods |
| WEBHOOK_MODE | Run bot in webhook mode or long-polling mode | `true` for webhook, 'false' for long-polling | `false` | All mods |
| PAPERS_DATABASE_URI | HTTP(S) URI with C++ proposals JSON file | Any valid URI | `https://wg21.link/index.json` | All mods |
| BIND_ADDRESS | Address for binding the web-service | Any valid IP address | `0.0.0.0` | Webhook mode |  
| BIND_PORT | Port for binding the web-service | Any valid port | `8080` | Webhook mode |
| HOST | Host, where Telegram will send updates in webhook mode | Any valid host address | None | Webhook mode |

If for any variable there is no default value and you didn't provide any value - the bot won't start.
Bot automatically registers webhook (if is launched in webhook mode) with address `https://$HOST/$TELOXIDE_TOKEN/api/v1/message`.

### How to use
Currently the only way to use the bot is inline mode. Write a C++ proposal number in any paired brackets (e.g. `[p1000]` or `{p1000}`) and the bot will return all corresponding results.

Search functionality isn't supported yet (WIP). Stay tuned :)

### Feedback
If you have any suggestions or want to report a bug - feel free to create in issue in this repo. Thank you!
