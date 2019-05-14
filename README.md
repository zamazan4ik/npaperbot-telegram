# npaperbot-telegram
Search C++ proposals in Telegram.

### Dependencies
* C++ compiler with C++17 support
* CMake + make/ninja/whatever else
* Conan

### How to build
* Clone this repository
* mkdir build && cd build
* `tgbot-cpp` you can get here: https://github.com/ZaMaZaN4iK/conan-tgbot-cpp
* `conan install ..`
* `cmake ..`
* `make` (if you use make)

### How to run
You must provide Telegram Bot API token to the `npaperbot-telegram` with `--token` option.

I recommend to run this bot as a service on a machine.