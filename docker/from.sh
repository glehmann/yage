#!/bin/sh

case "$1" in
    "linux/amd64") target='scratch' ;;
    "linux/arm64") target='scratch' ;;
    "linux/386") target='scratch' ;;
    "linux/arm/v7") target='scratch' ;;
    "linux/arm/v6") target='scratch' ;;
    "linux/ppc64le") target='debian:12-slim' ;;
    "linux/s390x") target='debian:12-slim' ;;
    *) echo >&2 "error: unsupported $TARGETPLATFORM architecture"; exit 1 ;;
esac;

echo -n $target
