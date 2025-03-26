FROM rust:1.85.1
ARG URL
WORKDIR /app
COPY . .
RUN DISCORD_WEBHOOK_URL=$URL cargo build --release
CMD ["/app/target/release/rss-read-kun"]
