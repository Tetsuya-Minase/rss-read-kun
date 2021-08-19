FROM rust:1.54.0
ARG URL
WORKDIR /app
COPY . .
RUN DISCORD_WEBHOOK_URL=$URL cargo build --release
CMD ["/app/target/release/rss-read-kun"]
