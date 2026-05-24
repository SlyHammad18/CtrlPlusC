// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "toggle" {
        ctrl_c_lib::send_toggle();
        return;
    }
    ctrl_c_lib::run()
}
