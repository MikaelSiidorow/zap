#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == "--toggle") {
        toggle_via_socket();
        return;
    }
    zap_lib::run();
}

fn toggle_via_socket() {
    use std::os::unix::net::UnixStream;

    let path = zap_lib::socket_path();
    match UnixStream::connect(&path) {
        Ok(_) => println!("Toggle signal sent"),
        Err(e) => {
            eprintln!("Failed to connect to zap: {e}. Is it running?");
            std::process::exit(1);
        }
    }
}
