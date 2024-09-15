#!/bin/bash

version=$1

if [ -z "$version" ]; then
  echo "Error: No version argument provided."
  exit 1
fi

# ARM64 darwin
cargo build --release

APP_NAME="lnrecc"
ARCHIVE_NAME="releases/${APP_NAME}-darwin-arm64-${version}.tar.gz"

tar -czvf $ARCHIVE_NAME -C target/release $APP_NAME

# AMD64 linux

BUILD_IMAGE_NAME="lnrecc-build"
ARCHIVE_NAME="releases/${APP_NAME}-linux-amd64-${version}.tar.gz"

docker build -f Dockerfile-build -t ${BUILD_IMAGE_NAME} --platform linux/amd64 .

container_id=$(docker run -d ${BUILD_IMAGE_NAME})

docker cp $container_id:/usr/src/app/target/release/$APP_NAME $APP_NAME

tar -czvf $ARCHIVE_NAME $APP_NAME

# cleanup

rm $APP_NAME
docker rm $container_id
docker rmi $BUILD_IMAGE_NAME