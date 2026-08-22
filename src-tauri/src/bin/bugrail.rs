// Keep the Bugrail desktop target aligned with the product-owned Cargo package
// name while retaining the inherited `codeg` binary and its credential-helper
// entrypoint.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The development desktop entrypoint can also be installed as Git's
    // credential helper, so it must honor the same subprocess contract as the
    // compatibility `codeg` binary.
    if std::env::args().any(|arg| arg == "--credential-helper") {
        let _log_guard = codeg_lib::logging::init::init_stderr_only();
        codeg_lib::git_credential::run_credential_helper();
        return;
    }

    codeg_lib::run();
}
