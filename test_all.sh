#!/usr/bin/env bash

set -e

for f in $(find . -type f -name Cargo.toml -print)
do
	echo $f
	cargo test --manifest-path=$f
done

echo SUCCESS!
