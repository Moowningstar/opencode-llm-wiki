FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy pre-compiled binary
COPY target/release/llm-wiki-server /usr/local/bin/llm-wiki-server

# Ensure binary is executable
RUN chmod +x /usr/local/bin/llm-wiki-server

# Expose API port
EXPOSE 19828

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:19828/health || exit 1

# Run the server
CMD ["llm-wiki-server", "--host", "0.0.0.0", "--port", "19828"]
