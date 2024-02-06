VERSION --global-cache 0.7
IMPORT github.com/earthly/lib/rust AS rust

ARG --global CROSS_VERSION=0.2.5

cross-deps:
    ARG NATIVEPLATFORM
    FROM --platform=$NATIVEPLATFORM rust:slim
    RUN apt-get update \
        && apt-get install -y docker.io jq wget \
        && apt-get clean
    WORKDIR /app
    DO rust+INIT --keep_fingerprints=true
    # DO rust+CARGO --args="install cross@${CROSS_VERSION}"
    RUN wget -O- "https://github.com/cross-rs/cross/releases/download/v${CROSS_VERSION}/cross-x86_64-unknown-linux-musl.tar.gz" | tar -xzf - -C /usr/local/bin
    DO rust+SET_CACHE_MOUNTS_ENV
    COPY docker/target.sh /
    COPY --keep-ts . ./
    DO rust+CARGO --args="fetch"

cross:
    FROM +cross-deps
    ARG TARGETPLATFORM
    ARG target=$(/target.sh $TARGETPLATFORM)
    # RUN rustup target add $target
    WITH DOCKER --pull ghcr.io/cross-rs/$target:$CROSS_VERSION
        RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE \
            --mount=$EARTHLY_RUST_TARGET_CACHE \
            rm -rf target/release \
            && cross build --target $target --release
    END
    DO rust+COPY_OUTPUT --output=".+/release/[^\./]+"
    SAVE ARTIFACT /app/target/$target/release/yage

docker-build:
    ARG from=scratch
    FROM $from
    WORKDIR /app
    COPY +cross/yage  /yage
    # make sure we have the required dependencies in the image
    # can't do that unfortunately because of https://github.com/earthly/earthly/issues/2618
    # RUN ["/yage", "--help"]
    ENTRYPOINT ["/yage"]
    ARG tag=main
    # SAVE IMAGE --push glehmann/yage:$tag
    SAVE IMAGE --push ghcr.io/glehmann/yage:$tag

docker:
    FROM alpine
    ARG TARGETPLATFORM
    COPY docker/from.sh /
    ARG from=$(/from.sh $TARGETPLATFORM)
    ARG tag=main
    BUILD +docker-build --from=$from --tag=main

docker-multiplatform:
    ARG tag=main
    BUILD \
        --platform=linux/amd64 \
        --platform=linux/arm64 \
        --platform=linux/386 \
        --platform=linux/arm/v7 \
        --platform=linux/arm/v6 \
        --platform=linux/ppc64le \
        --platform=linux/s390x \
        +docker --tag=$tag
