# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory
WORKDIR .

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock config.toml ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Expose the port your application uses (replace 8083 with your app's port)
EXPOSE 8080

# Set the unbuffered environment variable
ENV RUST_BACKTRACE "1"

# Run the binary
CMD ["./target/release/quest_server"]
