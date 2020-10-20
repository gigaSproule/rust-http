FROM liuchong/rustup as builder
RUN USER=root cargo new --bin rust-http
WORKDIR ./rust-http
COPY ./Cargo.toml ./Cargo.toml
RUN rustup default nightly
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/rust_http*
RUN cargo build --release
RUN pwd


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /root/rust-http/target/release/rust-http ${APP}/rust-http

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./rust-http"]
