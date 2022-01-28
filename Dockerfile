FROM rust:1.58.1-slim-bullseye AS builder

RUN apt-get update \
    && apt-get install --yes libncurses5-dev \
    && rm -fr /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /build/apple1

# hack to cache dependencies
COPY Cargo.toml Cargo.lock ./
# replace lib.rs with a dummy file to build an empty app with all dependencies
RUN sed -i 's/src\/lib.rs/dummy1.rs/' Cargo.toml
RUN sed -i 's/src\/bin.rs/dummy2.rs/' Cargo.toml
RUN echo 'fn main() {}' > dummy1.rs
RUN echo 'fn main() {}' > dummy2.rs
RUN cargo build --release --features binary

# replace dummy.rs back
RUN sed -i 's/dummy1.rs/src\/lib.rs/' Cargo.toml
RUN sed -i 's/dummy2.rs/src\/bin.rs/' Cargo.toml

# copy source files
COPY src/ ./src
# build the binary
RUN cargo build --release --features binary

FROM ubuntu:20.04

WORKDIR /app/
COPY --from=builder /build/apple1/target/release/apple1 .

COPY asm /app/asm
COPY sys /app/sys
COPY roms /app/roms

CMD ["/app/apple1"]
