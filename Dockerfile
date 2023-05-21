FROM rust:slim-buster
RUN apt-get update -y && apt-get upgrade -y
RUN apt-cache policy sqlite3
RUN apt-get install -y sqlite3 libsqlite3-dev
WORKDIR /usr/src/app

COPY . .
RUN cargo build --release

EXPOSE 8080
ENTRYPOINT while true; do ./target/release/haxmail "0.0.0.0:8080"; done