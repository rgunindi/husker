FROM ubuntu:latest AS chef

# Install Rust and cargo-chef dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install build dependencies
RUN apt-get update && apt-get install -y wget build-essential \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# Install `dx`
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force

ENV PATH="/.cargo/bin:${PATH}"

# Create the final bundle folder
RUN dx bundle --platform server

FROM ubuntu:22.04 AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates && apt-get clean && rm -rf /var/lib/apt/lists/*

# Sertifika yollarını güncelle
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

COPY --from=builder /app/target/dx/husker/release/web/ /usr/local/app

# Set environment variables
ENV PORT=8080
ENV IP=0.0.0.0

# Expose the port 8080
EXPOSE 8080

WORKDIR /usr/local/app
ENTRYPOINT ["/usr/local/app/server", "--port", "${PORT}"]

