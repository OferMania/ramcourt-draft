#!/bin/bash
set -eux

CLSTR_BRANCH=${CLSTR_BRANCH:-"main"}
rm -rf clstr-rc
git clone --branch ${CLSTR_BRANCH} https://github.com/OferMania/clstr-rc
cd clstr-rc
go build -o clstr ./cmd/clstr
cp clstr ../
cd ..
