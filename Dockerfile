FROM rust:1.69 as builder

# Build & cache dependencies
WORKDIR /
RUN USER=root cargo new chat
COPY Cargo.lock Cargo.toml /chat/
WORKDIR /chat
RUN cargo build --release

# Build source
COPY src ./src
RUN cargo build --release

# Final image
FROM gcr.io/distroless/cc

COPY --from=builder /chat/target/release/chat ./
COPY static ./static

ENTRYPOINT [ "./chat" ]