REM
cargo build

.\target\debug\snapmail-cli.exe alex clear
.\target\debug\snapmail-cli.exe billy clear
.\target\debug\snapmail-cli.exe camille clear

.\target\debug\snapmail-cli.exe alex setup testnet --dna-path build\snapmail-rsm\snapmail.dna network -b https://bootstrap-staging.holo.host/ quic
.\target\debug\snapmail-cli.exe billy setup testnet --dna-path build\snapmail-rsm\snapmail.dna network -b https://bootstrap-staging.holo.host/ quic
.\target\debug\snapmail-cli.exe camille setup testnet --dna-path build\snapmail-rsm\snapmail.dna network -b https://bootstrap-staging.holo.host/ quic


REM set WASM_LOG=ERROR
REM set RUST_LOG=ERROR
REM .\target\release\snapmail-cli.exe alex setup testnet --dna-path ../snapmail-release/dna/snapmail.dna network -b https://bootstrap-staging.holo.host/ quic
REM .\target\debug\snapmail-cli.exe alex setup testnet --dna-path ../snapmail-rsm/snapmail.dna network -b https://bootstrap-staging.holo.host/ quic


start cmd /c .\target\debug\snapmail-tui.exe alex
start cmd /c .\target\debug\snapmail-tui.exe billy
start cmd /c .\target\debug\snapmail-tui.exe camille
