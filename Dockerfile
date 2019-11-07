FROM fedora:latest

LABEL maintainer="Alexander Zaitsev <zamazan4ik@tut.by>"

ARG NPAPERBOT_VERSION=1.0.0

ARG NPAPERBOT_SOURCE_URL=https://github.com/ZaMaZaN4iK/npaperbot-telegram/archive/v${NPAPERBOT_VERSION}.zip

RUN dnf install -y perl autoconf automake libtool wget unzip clang cmake make python3-pip \
    && pip install conan

RUN conan remote add zamazan4ik https://api.bintray.com/conan/zamazan4ik/conan

WORKDIR /npaperbot_telegram

RUN wget https://github.com/ZaMaZaN4iK/npaperbot-telegram/archive/v${NPAPERBOT_VERSION}.zip && unzip v1.0.0.zip

WORKDIR npaperbot-telegram-1.0.0

RUN mkdir build

WORKDIR build

RUN CC=clang CXX=clang++ conan install .. --build=missing && CC=clang CXX=clang++ cmake .. -DCMAKE_BUILD_TYPE=Release && make

WORKDIR bin

RUN chmod +x ./npaperbot_telegram
RUN mkdir logs

ENTRYPOINT ["/npaperbot_telegram/npaperbot-telegram-1.0.0/build/bin/npaperbot_telegram"]

