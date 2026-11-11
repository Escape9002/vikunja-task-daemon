# Stage 1: Build the binary statically
FROM rust:1.99-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Create a dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Copy real source code and build
COPY src ./src
# Touch main.rs to ensure Cargo recompiles it
RUN touch src/main.rs && cargo build --release

# Stage 2: Final lightweight runtime
FROM alpine:latest

# Install CA certificates and Timezone data
RUN apk add --no-cache ca-certificates tzdata
RUN addgroup -g 10001 daemon-group && \
    adduser -u 10001 -G daemon-group -h /app -D daemon-user

WORKDIR /app
COPY --from=builder /app/target/release/vikunja-task-daemon .

RUN mkdir -p /etc/vikunja-daemon && chown daemon-user:daemon-group /etc/vikunja-daemon

USER daemon-user
CMD ["./vikunja-task-daemon"]