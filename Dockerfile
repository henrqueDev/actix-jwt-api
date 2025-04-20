# Use a Rust base image with Cargo installed
FROM rust:1.83.0 AS builder

RUN apt-get install -y pkg-config && \
    apt-get install -y libssl-dev

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

RUN cargo install diesel_cli --no-default-features --features postgres

COPY . ./

# Build the dependencies without the actual source code to cache dependencies separately
RUN cargo build --release

EXPOSE 8080
EXPOSE 587

# Command to run the application
CMD ["./target/release/actix-jwt-api"]