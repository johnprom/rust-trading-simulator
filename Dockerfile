# ========= 1) Base image with Rust =========
FROM rust:1.89-slim as builder

# Set working directory inside container
WORKDIR /app

# Install system dependencies for Rust + OpenSSL
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Install Dioxus CLI
RUN cargo install dioxus-cli --locked

# Copy entire project
COPY . .

# --- Build frontend ---
WORKDIR /app/frontend
RUN dx build --release

# Create a place for static files in backend
WORKDIR /app/backend
RUN mkdir -p static

# Copy frontend dist output to backend/static
# Adjust the source path if your dx build output directory differs
RUN cp -r /app/frontend/dist/* /app/backend/static/ || true

# --- Build backend in release mode ---
RUN cargo build --release

# ========= 2) Runtime image (smaller) =========
FROM debian:bookworm-slim

# Install system dependencies for Rust binaries (OpenSSL, etc.)
RUN apt-get update && \
    apt-get install -y libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy only the built backend binary and static files
COPY --from=builder /app/backend/target/release/backend /app/backend
COPY --from=builder /app/backend/static /app/static

# Expose the port your Axum server listens on (change if different)
EXPOSE 3000

# Run the backend binary
CMD ["./backend"]
