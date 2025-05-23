VERSION --global-cache 0.8
IMPORT github.com/earthly/lib/rust AS rust

ARG --global CROSS_VERSION=0.2.5

cross-deps:
    ARG NATIVEPLATFORM
    FROM --platform=$NATIVEPLATFORM rust:1.84-slim
    RUN apt-get update \
        && apt-get install -y docker.io jq wget \
        && apt-get clean
    WORKDIR /app
    DO rust+INIT --keep_fingerprints=true
    # DO rust+CARGO --args="install cross@${CROSS_VERSION}"
    RUN wget -nv -O- "https://github.com/cross-rs/cross/releases/download/v${CROSS_VERSION}/cross-x86_64-unknown-linux-musl.tar.gz" | tar -xzf - -C /usr/local/bin
    DO rust+SET_CACHE_MOUNTS_ENV
    COPY --keep-ts . ./
    DO rust+CARGO --args=fetch

cross:
    FROM +cross-deps
    ARG TARGETPLATFORM
    LET target="unsupported platform"
    IF [ "$TARGETPLATFORM" = "linux/amd64" ]
        SET target=x86_64-unknown-linux-musl
    ELSE IF [ "$TARGETPLATFORM" = "linux/arm64" ]
        SET target=aarch64-unknown-linux-musl
    ELSE IF [ "$TARGETPLATFORM" = "linux/386" ]
        SET target=i686-unknown-linux-musl
    ELSE IF [ "$TARGETPLATFORM" = "linux/arm/v7" ]
        SET target=armv7-unknown-linux-musleabihf
    ELSE IF [ "$TARGETPLATFORM" = "linux/arm/v6" ]
        SET target=arm-unknown-linux-musleabihf
    ELSE IF [ "$TARGETPLATFORM" = "linux/ppc64le" ]
        SET target=powerpc64le-unknown-linux-gnu
    ELSE IF [ "$TARGETPLATFORM" = "linux/s390x" ]
        SET target=s390x-unknown-linux-gnu
    END
    DO rust+CROSS --target=$target
    DO rust+COPY_OUTPUT --output=".+/release/[^\./]+"
    SAVE ARTIFACT /app/target/$target/release/yage

docker-build:
    ARG from=scratch
    FROM $from
    WORKDIR /src
    COPY +cross/yage  /yage
    # make sure we have the required dependencies in the image
    # can't do that unfortunately because of https://github.com/earthly/earthly/issues/2618
    # RUN ["/yage", "--help"]
    ENTRYPOINT ["/yage"]
    ARG tag=main
    LABEL org.opencontainers.image.vendor="Gaëtan Lehmann" \
        org.opencontainers.image.url="https://github.com/glehmann/yage" \
        org.opencontainers.image.title="yage" \
        org.opencontainers.image.description="A simple tool to manage encrypted secrets in YAML files with age encryption." \
        org.opencontainers.image.version="${tag}" \
        org.opencontainers.image.documentation="https://github.com/glehmann/yage"
    # SAVE IMAGE --push glehmann/yage:$tag
    SAVE IMAGE --push ghcr.io/glehmann/yage:$tag

# we need a shell in the image in order to run a IF, so we run the IF
# in that image and delegate the actual image creation to docker-build
# with the FROM value as argument
docker:
    FROM alpine
    ARG TARGETPLATFORM
    ARG tag=main
    IF [ "$TARGETPLATFORM" = "linux/s390x" ] || [ "$TARGETPLATFORM" = "linux/ppc64le" ]
        # these platform are not statically linked, they can't run in a scratch image
        BUILD +docker-build --from=+debian-minimal --tag=$tag
    ELSE
        BUILD +docker-build --from=scratch --tag=$tag
    END

debian-deps:
    FROM debian:12-slim
    ARG TARGETPLATFORM
    IF [ "$TARGETPLATFORM" = "linux/ppc64le" ]
        WORKDIR /lib/powerpc64le-linux-gnu/
        RUN mkdir -p /deps/lib/powerpc64le-linux-gnu/ /deps/lib64/ \
            && cp libgcc_s.so.1 /deps/lib/powerpc64le-linux-gnu/ \
            && cp libpthread.so.0 /deps/lib/powerpc64le-linux-gnu/ \
            && cp libc.so.6 /deps/lib/powerpc64le-linux-gnu/ \
            && cp ld64.so.2 /deps/lib64/
    ELSE IF [ "$TARGETPLATFORM" = "linux/s390x" ]
        WORKDIR /lib/s390x-linux-gnu/
        RUN mkdir -p /deps/lib/s390x-linux-gnu/ /deps/lib/ \
            && cp libgcc_s.so.1 /deps/lib/s390x-linux-gnu/ \
            && cp libpthread.so.0 /deps/lib/s390x-linux-gnu/ \
            && cp libc.so.6 /deps/lib/s390x-linux-gnu/ \
            && cp librt.so.1 /deps/lib/s390x-linux-gnu/ \
            && cp ld64.so.1 /deps/lib/
    END
    SAVE ARTIFACT /deps

debian-minimal:
    FROM scratch
    COPY +debian-deps/* /

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
