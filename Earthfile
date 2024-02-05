VERSION 0.7

rust:
    FROM rust:alpine
    RUN apk add --no-cache musl-dev
    WORKDIR app
    RUN --mount=type=cache,target=/root/.cargo \
        cargo install cargo-chef

planner:
    FROM +rust
    COPY . ./
    RUN --mount=type=cache,target=/root/.cargo \
        cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

deps:
    FROM +rust
    COPY +planner/recipe.json recipe.json
    RUN cargo chef cook --release --recipe-path recipe.json

build:
    FROM +deps
    COPY . ./
    RUN --mount=type=cache,target=/root/.cargo \
        cargo build --release
    SAVE ARTIFACT target/release/yage

docker:
    FROM scratch
    WORKDIR /app
    COPY +build/yage /yage
    ENTRYPOINT ["/yage"]
    ARG tag=latest
    SAVE IMAGE --push glehmann/yage:$tag
