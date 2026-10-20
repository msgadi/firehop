// Keeps release builds from opening a console window next to the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    firehop_lib::run();
}
