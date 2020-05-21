FROM rust:1.43.1-slim-buster AS builder

RUN apt-get update \
    && apt-get install --yes libncurses5-dev \
    && rm -fr /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /build/apple1

# hack to cache dependencies
COPY Cargo.toml Cargo.lock ./
# replace main.rs with a dummy file to build an empty app with all dependencies
RUN sed -i 's/src\/main.rs/dummy.rs/' Cargo.toml
RUN echo 'fn main() {}' > dummy.rs
RUN cargo build --release
# replace dummy.rs with src/main.rs to buils the app
RUN sed -i 's/dummy.rs/src\/main.rs/' Cargo.toml

# copy source files
COPY src/ ./src
# build the binary
RUN cargo build --release

FROM ubuntu:20.04

WORKDIR /app/
COPY --from=builder /build/apple1/target/release/apple1 .

COPY asm /app/asm
COPY sys /app/sys
COPY roms /app/roms

CMD ["/app/apple1"]
