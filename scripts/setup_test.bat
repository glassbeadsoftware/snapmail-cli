REM
cargo build

.\target\debug\snapmail-cli.exe alex clear
.\target\debug\snapmail-cli.exe billy clear
.\target\debug\snapmail-cli.exe camille clear

.\target\debug\snapmail-cli.exe alex setup testnet network -b https://bootstrap-staging.holo.host/ quic
.\target\debug\snapmail-cli.exe billy setup testnet network -b https://bootstrap-staging.holo.host/ quic
.\target\debug\snapmail-cli.exe camille setup testnet network -b https://bootstrap-staging.holo.host/ quic

start cmd /c .\target\debug\snapmail-tui.exe alex
start cmd /c .\target\debug\snapmail-tui.exe billy
start cmd /c .\target\debug\snapmail-tui.exe camille
