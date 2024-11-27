FROM debian:bookworm-slim
ENV SHELL /bin/bash
ARG user_id=1000
ARG group_id=1000

# Install packages
RUN apt-get update --fix-missing && DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    procps \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Install binaryen (for wasm-opt)
RUN curl -fsSL https://github.com/WebAssembly/binaryen/releases/download/version_119/binaryen-version_119-x86_64-linux.tar.gz | \
    tar xvz --strip-components 1 --directory /usr

# Create a developer user
RUN groupadd --gid $group_id developer && useradd --uid $user_id --gid $group_id -m developer
# Switch the developer user
USER developer

# install Rust (make sure toolchain file matches the nightly version used here)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target wasm32-unknown-unknown
ENV PATH=/home/developer/.cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

RUN cargo binstall --no-confirm wasm-bindgen-cli@0.2.95
