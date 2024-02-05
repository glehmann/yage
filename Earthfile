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
