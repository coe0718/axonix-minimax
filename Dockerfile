FROM rust:1.85

RUN apt-get update && apt-get install -y \
    git \
    python3 \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update && apt-get install -y gh \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 1000 --create-home axonix

RUN git config --system --add safe.directory /workspace \
    && git config --system user.email "axonix@axonix.live" \
    && git config --system user.name "Axonix"

ENV CARGO_INCREMENTAL=0

WORKDIR /workspace

# Cache dependencies before copying real source
COPY Cargo.toml Cargo.lock* ./
COPY vendor/ ./vendor/
RUN mkdir -p src src/bin \
    && echo 'fn main() {}' > src/main.rs \
    && echo 'fn main() {}' > src/bin/stream_server.rs \
    && cargo build \
    && rm -rf src \
       target/debug/axonix \
       target/debug/stream_server \
       target/debug/deps/axonix-* \
       target/debug/deps/stream_server-*

# Build the real project
COPY . .
RUN cargo build
