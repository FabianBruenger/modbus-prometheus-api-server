#
# This Dockerfile is ment to be used on a Github runner with a Ubuntu 20.04 OS
# Therefor it is corss-compiled from a linux machine to a Docker Image for MacOS M1
#
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt -y update
RUN apt install -y musl-tools musl-dev 
RUN apt-get install -y build-essential libssl-dev pkg-config
RUN apt install -y gcc-x86-64-linux-gnu
WORKDIR /app
RUN mkdir config

# Copy files & dirs selectivly
COPY ./Cargo.toml .
COPY ./src ./src
COPY ./docker/modbus-prometheus-api-server/setup-prod.toml ./setup.toml

# For a musl build on M1 Macs, these ENV variables have to be set
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC='gcc'
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc
ENV CC_x86_64-unknown-linux-musl=x86_64-linux-gnu-gcc
# Build macOSM1 binary
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/modbus-prometheus-api-server ./
COPY --from=builder /app/setup.toml ./
COPY ./config/test_client.json ./config/test_client.json
CMD ["/app/modbus-prometheus-api-server"]
