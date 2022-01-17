#!/bin/sh

dir=$(dirname "$0")
cd "$dir" || exit 1
echo "executing build from $(pwd)"

$(./get_cri.sh) build -f ./Dockerfile -t "${TAG:-zk-test-harness}" ../..
