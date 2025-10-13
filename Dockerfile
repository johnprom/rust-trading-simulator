FROM rust:1.82-slim AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build frontend
WORKDIR /app/frontend
COPY frontend/Cargo.toml frontend/Cargo.lock* ./
COPY frontend/src ./src
RUN cargo install dioxus-cli --locked
RUN dx build --release

# Build backend
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/src ./src
RUN cargo build --release

# Runtime
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/backend/target/release/backend ./backend
COPY --from=builder /app/frontend/target/dx/frontend/release/web/public ./static

EXPOSE 3000
CMD ["./backend"]

# # ===== Stage 1: Build =====
# FROM rust:1.89-slim AS builder

# WORKDIR /app

# # Install dependencies for building
# RUN apt-get update && \
#     apt-get install -y pkg-config libssl-dev && \
#     rm -rf /var/lib/apt/lists/*

# # Copy project
# COPY . .

# # Build frontend
# WORKDIR /app/frontend
# RUN cargo install dioxus-cli --locked
# RUN dx build --release

# # Copy frontend output to backend static directory
# WORKDIR /app/backend
# RUN mkdir -p static
# RUN cp -r /app/frontend/target/dx/frontend/release/web/public/* static/ || true

# # Build backend
# RUN cargo build --release

# # ===== Stage 2: Runtime =====
# FROM debian:bookworm-slim

# WORKDIR /app/backend

# # Install SSL lib for runtime
# RUN apt-get update && \
#     apt-get install -y libssl-dev && \
#     rm -rf /var/lib/apt/lists/*

# # Copy built backend
# COPY --from=builder /app/backend/target/release/backend .

# # Copy static files
# COPY --from=builder /app/backend/static ./static

# # Expose port and run
# EXPOSE 3000
# CMD ["./backend"]
