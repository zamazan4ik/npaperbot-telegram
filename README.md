# npaperbot-telegram
Search C++ proposals in Telegram.

### Dependencies
* C++ compiler with C++17 support
* CMake + make/ninja/whatever else
* Conan

### How to build
* Clone this repository
* mkdir build && cd build
* `tgbot-cpp` you can get here: https://github.com/ZaMaZaN4iK/conan-tgbot-cpp. Since this package isn't currently available on any Conan remote - clone the 'conan-tgbot-cpp' to a build machine, install it to a local cache. It makes tgbot-cpp available for the next build step.
* `conan install ..`
* `cmake ..`
* `make` (if you use make)

### How to run
You must provide Telegram Bot API token to the `npaperbot-telegram` with `--token` option. `npaperbot-telegram` has other command line options but only `--token` is mandatory - other options have some reasonable defaults.

So your command line for running `npaperbot-telegram` can be like this one:
`npaperbot-telegram --token ${TOKEN} --log-path ${LOG_PATH}`

I recommend to run this bot as a service(e.g. as systemd service) on a machine.
