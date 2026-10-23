#!/bin/bash

cd doc_gen/bin
cargo run -- gen-self --docs-documentation-template
mkdir -p ../../addons
mkdir -p ../../addons/tealr_doc_gen
unzip -o ./pages/definitions/tealr_doc_gen.zip -d ../../addons/tealr_doc_gen
cd ../../