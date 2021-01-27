FROM rust:1.49-buster AS build
WORKDIR /src
COPY app/Cargo.lock app/Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
COPY app/* ./
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get -y install libpq5
RUN apt-get install -y ca-certificates
RUN update-ca-certificates
COPY --from=build /src/target/release/app .
CMD ["./app"]
