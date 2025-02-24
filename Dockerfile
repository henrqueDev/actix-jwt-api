# Use a Rust base image with Cargo installed
FROM rust:1.83.0 AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./


COPY . ./
# # Create an empty src directory to trick Cargo into thinking it's a valid Rust project
# RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies without the actual source code to cache dependencies separately
RUN cargo build --release


# Now copy the source code

# Build your application

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM debian:bookworm-slim

# Install libpq for PostgreSQL connectivity
RUN apt-get update && \
    apt-get install -y libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the built binary from the previous stage
COPY --from=builder /app/target/release/pethotel-api ./

EXPOSE 8080

# Command to run the application
CMD ["./pethotel-api"]