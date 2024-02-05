VERSION --global-cache 0.7
IMPORT github.com/earthly/lib/rust AS rust

rust:
    FROM rust:alpine
    RUN apk add --no-cache musl-dev findutils
    DO rust+INIT --keep_fingerprints=true
    DO rust+SET_CACHE_MOUNTS_ENV

build:
    FROM +rust
    COPY --keep-ts . ./
    DO rust+CARGO --args="build --release" --output="release/[^/\.]+"
    SAVE ARTIFACT target/release/yage

prebuilt:
    LOCALLY
    ARG --required platform_slug
    ARG --required target
    RUN mkdir -p bin \
        && cp target/$target/release/yage bin/yage-$platform_slug

cross:
    LOCALLY
    ARG --required platform_slug
    ARG --required target
    RUN cross build --target $target --release  \
        && cp target/$target/release/yage bin/yage-$platform_slug

cross-dind:
    FROM rust:alpine
    RUN apk add --no-cache musl-dev findutils
    DO rust+INIT --keep_fingerprints=true
    DO rust+SET_CACHE_MOUNTS_ENV
    DO rust+CARGO --args="install cross"
    RUN apk add docker jq
    ARG --required target
    COPY --keep-ts . ./
    WITH DOCKER
        RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE \
            --mount=$EARTHLY_RUST_TARGET_CACHE \
            cross build --target $target --release
    END
    DO rust+COPY_OUTPUT --output="[^\./]+/release/[^\./]+"

artifact:
    FROM alpine
    ARG TARGETPLATFORM
    RUN echo "Building for $TARGETPLATFORM"
    ARG platform_slug=$(echo $TARGETPLATFORM | tr / -)
    ARG build=earthly
    COPY platform.sh /platform.sh
    ARG target=$(/platform.sh $TARGETPLATFORM)
    IF [ "$build" = "earthly" ]
        COPY +build/yage /yage
    ELSE IF [ "$build" = "prebuilt" ]
        WAIT
            BUILD +prebuilt --platform_slug=$platform_slug --target=$target
        END
        COPY bin/yage-$platform_slug /yage
    ELSE IF [ "$build" = "cross" ]
        WAIT
            BUILD +cross --platform_slug=$platform_slug --target=$target
        END
        COPY bin/yage-$platform_slug /yage
    ELSE IF [ "$build" = "cross-dind" ]
        WAIT
            ARG NATIVEPLATFORM
            BUILD --platform $NATIVEPLATFORM +cross --target=$target
        END
        COPY bin/yage-$platform_slug /yage
    END
    SAVE ARTIFACT yage

docker:
    FROM scratch
    WORKDIR /app
    ARG build=earthly
    COPY (+artifact/yage --build=$build) /yage
    ENTRYPOINT ["/yage"]
    ARG tag=latest
    SAVE IMAGE --push glehmann/yage:$tag

docker-multiplatform:
    ARG build=earthly
    BUILD \
        --platform=linux/amd64 \
        --platform=linux/arm64 \
        --platform=linux/386 \
        --platform=linux/arm/v7 \
        --platform=linux/arm/v6 \
        --platform=linux/ppc64le \
        --platform=linux/s390x \
        +docker --build=$build
