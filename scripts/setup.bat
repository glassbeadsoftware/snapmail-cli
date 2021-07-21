REM Script for building wasm and rustifying it

REM Create build Directory
rmdir /S /Q build
mkdir build
cd build

REM Generate WASM
git clone https://github.com/glassbeadsoftware/snapmail-rsm

cd snapmail-rsm
cargo build --release --target wasm32-unknown-unknown
cd ..

REM Done
cd ..
