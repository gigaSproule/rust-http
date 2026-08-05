FROM liuchong/rustup:nightly@sha256:fd793d6f4f1bfc58c75f8e5e60356521d4d8e7617ebfa9a9c88869ccb272d60f as builder
RUN USER=root cargo new --bin rust-http
WORKDIR ./rust-http
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/rust_http*
RUN cargo build --release
RUN pwd


FROM debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241
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
