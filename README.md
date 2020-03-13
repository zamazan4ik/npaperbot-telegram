# npaperbot-telegram
Search C++ proposals in Telegram.

### Dependencies
* C++ compiler with C++17 support
* CMake
* Conan

### How to build
* Clone this repository
* `cmake -B build`
* `cmake --build build --target install`

### How to run
You must provide Telegram Bot API token to the `npaperbot-telegram` with `--token` option. `npaperbot-telegram` has other command line options but only `--token` is mandatory - other options have some reasonable defaults.

So your command line for running `npaperbot-telegram` can be like this one:
`npaperbot_telegram --token ${TOKEN} --log-path ${LOG_PATH}`

I recommend to run this bot as a service(e.g. as systemd service) on a machine.
Also Docker images are available here: https://hub.docker.com/repository/docker/zamazan4ik/npaperbot-telegram
