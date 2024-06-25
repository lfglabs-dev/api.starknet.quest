# Use the official Rust image as the base image
FROM rust:1.73.0

# Set the working directory
WORKDIR .

# Copy the Cargo.toml and config files
COPY Cargo.toml config.toml ./

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
