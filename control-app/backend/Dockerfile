FROM debian:bookworm

# Basic packages
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
    && rm -rf /var/lib/apt/lists/*

# Toolchain
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        pkg-config \
        build-essential\
        cmake \
        ninja-build \
        make \
        nodejs \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        binutils-aarch64-linux-gnu \
        gcc-aarch64-linux-gnu \
        libc6-dev-arm64-cross \
    && curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal \
    &&  . /root/.cargo/env && rustup target add aarch64-unknown-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Dependencies
RUN dpkg --add-architecture arm64 \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        libglib2.0-dev:arm64 \
        libpcre2-dev:arm64 \
        libffi-dev:arm64 \
        zlib1g-dev:arm64 \
        libmount-dev:arm64 \
        libgstreamer1.0-dev:arm64 \
        libgstreamer-plugins-base1.0-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

ENV CMAKE_GENERATOR=Ninja
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar \
    PKG_CONFIG=/usr/bin/pkg-config \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig \
    PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig \
    PKG_CONFIG_STATIC=1 \
    PKG_CONFIG_ALL_STATIC=1 \
    TURBOJPEG_STATIC=1
