FROM liuchong/rustup
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
ADD src /app
WORKDIR /app
RUN rustup default nightly
RUN cargo build
CMD ["cargo", "run"]
