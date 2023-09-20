FROM rust:slim as build
WORKDIR /usr/src/s3-utils
COPY . .
RUN apt-get update && apt install -y pkg-config libssl-dev
RUN cargo install --path .
RUN cargo build --release

# Copy artefacts to a clean image
FROM rust:slim
RUN apt-get update && apt install -y openssl pkg-config libssl-dev
COPY --from=build /usr/src/s3-utils/target/release/s3-utils /usr/local/bin/release/s3-utils