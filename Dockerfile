FROM rust:1.67 as build
WORKDIR /usr/src/s3-utils
COPY . .
RUN cargo install --path .
RUN cargo build --release

# Copy artefacts to a clean image
FROM debian:buster-slim
RUN apt-get update && apt install -y openssl
COPY --from=build /usr/src/s3-utils/target/release/s3-utils /usr/local/bin/release/s3-utils