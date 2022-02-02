#!/bin/sh

## !! Provide new REV as first argument !!

# Extract OLD REV from holochain_rev.txt
value=`cat holochain_rev.txt`
echo "OLD REV = '$value'"
echo "NEW REV = '$1'"

# Replace REV in sweettest/Cargo.toml
sed -i "s/$value/$1/g" sweettest/Cargo.toml

# Replace REV in zomes/snapmail/Cargo.toml
sed -i "s/$value/$1/g" zomes/snapmail/Cargo.toml

# Replace REV in wasm_utils/Cargo.toml
sed -i "s/$value/$1/g" crates/wasm_utils/Cargo.toml

# Replace REV in dna_wasm/Cargo.toml
sed -i "s/$value/$1/g" crates/dna_wasm/Cargo.toml

# Replace REV in holochain_rev.txt
sed -i "s/$value/$1/g" holochain_rev.txt
