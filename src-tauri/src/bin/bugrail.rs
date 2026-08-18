// Keep the Bugrail desktop target aligned with the product-owned Cargo package
// name while retaining the inherited `codeg` binary and its credential-helper
// entrypoint.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    codeg_lib::run();
}
