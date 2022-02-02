REM Build and Rustify Snapmail WASM
cargo build --release -p rustify_wasm

.\target\release\rustify-wasm.exe build\snapmail-rsm\target\wasm32-unknown-unknown\release\snapmail.wasm
