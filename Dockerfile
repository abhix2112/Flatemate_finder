# Step 1: Builder
FROM rust:1.77 as builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy full source
COPY . .

# Build real project
RUN cargo build --release

# Step 2: Runtime
FROM debian:bullseye-slim

# Install libssl (needed by Rust SQLx with TLS)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Create app user (optional)
RUN useradd -m appuser

WORKDIR /app
COPY --from=builder /app/target/release/backend .

# Copy migrations folder if needed at runtime
COPY ./migrations ./migrations

# Use non-root user (optional but recommended)
USER appuser

ENV RUST_LOG=info
EXPOSE 8080

CMD ["./backend"]
