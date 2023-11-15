# Integration tests for the Template Wallet.
# Requires a `cargo build` to be run before.

./target/debug/node-template --dev &
sleep 20 &&
./target/debug/tuxedo-template-wallet --dev
