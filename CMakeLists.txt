cmake_minimum_required(VERSION 3.14)
project(npaperbot_telegram)

set(CMAKE_CXX_STANDARD 17)

add_compile_definitions(HAVE_CURL)

# Download automatically, you can also just copy the conan.cmake file
if(NOT EXISTS "${CMAKE_BINARY_DIR}/conan.cmake")
    message(STATUS "Downloading conan.cmake from https://github.com/conan-io/cmake-conan")
    file(DOWNLOAD "https://github.com/conan-io/cmake-conan/raw/v0.15/conan.cmake"
            "${CMAKE_BINARY_DIR}/conan.cmake")
endif()

include(${CMAKE_BINARY_DIR}/conan.cmake)

conan_cmake_run(REQUIRES
        zlib/[>1.2.0]@conan/stable
        boost/[>1.70.0]@conan/stable
        cli11/[>1.3.0]@bincrafters/stable
        jsonformoderncpp/[>3.6.1]@vthiery/stable
        spdlog/[>1.3.1]@bincrafters/stable
        tgbot/[>1.0]
        BASIC_SETUP
        BASIC
        BUILD missing)

add_executable(npaperbot_telegram main.cpp)
target_link_libraries(npaperbot_telegram ${CONAN_LIBS})