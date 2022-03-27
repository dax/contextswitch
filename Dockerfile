FROM lukemathwalker/cargo-chef:latest-rust-1.57.0 as chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
RUN cargo install cargo-make
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook -p contextswitch --release --recipe-path recipe.json
RUN cargo chef cook -p contextswitch-api --release --recipe-path recipe.json
RUN cargo chef cook -p contextswitch-web --release --recipe-path recipe.json --target wasm32-unknown-unknown
COPY . .
RUN cargo make build-release
RUN sed -i 's#http://localhost:8000/api#/api#' web/dist/snippets/contextswitch-web-*/js/api.js

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN mkdir /data
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl taskwarrior \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/contextswitch-api contextswitch
COPY --from=builder /app/api/config/default.toml config/default.toml
COPY --from=builder /app/web/dist/ .
ENV CS_TASKWARRIOR.DATA_LOCATION /data
ENV CS_APPLICATION.API_PATH /api
ENV CS_APPLICATION.STATIC_PATH /
CMD ["/app/contextswitch"]
