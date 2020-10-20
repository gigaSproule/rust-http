FROM liuchong/rustup
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
ADD Cargo.toml /app/Cargo.toml
ADD src /app/src
WORKDIR /app
RUN rustup default nightly
RUN cargo build
CMD ["cargo", "run"]
