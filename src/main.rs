mod audioplayer;
mod pan;

use awedio::Sound;

fn main() {
    let mut manager = awedio::start().unwrap();
    let player = audioplayer::AudioPlayer::new(1.0, 1.0, 0.5, "test.mp3".to_string());
    player.play(&mut manager.0);
    std::thread::sleep(std::time::Duration::from_secs(100));
}
