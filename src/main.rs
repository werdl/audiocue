// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// filepath: /home/werdl/coding/audiocue/src-tauri/src/main.rs

mod audioplayer;
mod pan;


fn main() {
    let mut manager = awedio::start().unwrap();
    let player = audioplayer::AudioPlayer::new(1.0, 1.0, -1.0, 0.0,  "test.mp3".to_string());
    player.play_blocking(&mut manager.0);
}