REM Build and Rustify Snapmail WASM
cargo build

.\target\debug\rustify-wasm.exe build\snapmail-rsm\target\wasm32-unknown-unknown\release\snapmail.wasm

cargo build
