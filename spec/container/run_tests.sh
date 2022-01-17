#!/bin/sh

dir=$(dirname "$0")
cd "$dir" || exit 1
echo "executing run from $(pwd)"

$(./get_cri.sh) run -ti --rm -w /work -v ../..:/work:ro -e ZK_BIN=/work/zk "${TAG:-zk-test-harness}"
