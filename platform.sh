#!/bin/sh

case "$1" in
    "linux/amd64") target='x86_64-unknown-linux-musl' ;;
    "linux/arm64") target='aarch64-unknown-linux-musl' ;;
    "linux/386") target='i686-unknown-linux-musl' ;;
    "linux/arm/v7") target='armv7-unknown-linux-musleabihf' ;;
    "linux/arm/v6") target='arm-unknown-linux-musleabihf' ;;
    "linux/ppc64le") target='powerpc64le-unknown-linux-gnu' ;;
    "linux/s390x") target='s390x-unknown-linux-gnu' ;;
    *) echo >&2 "error: unsupported $TARGETPLATFORM architecture"; exit 1 ;;
esac;

echo -n $target
