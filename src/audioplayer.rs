use awedio::{
    manager, sounds::{
        self,
        wrappers::{AdjustableSpeed, AdjustableVolume, Controllable, Pausable, SetPaused, SetSpeed, SetVolume, Wrapper},
        Silence,
    }, Sound
};

use crate::pan::{Pan, Panned, SetPan};

pub struct AudioPlayer {
    controller: Controllable<AdjustableSpeed<AdjustableVolume<Panned<Pausable<Box<dyn Sound>>>>>>,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("volume", &self.controller.inner().inner().volume())
            .field("speed", &self.controller.inner().speed())
            .field(
                "frames",
                &format!(
                    "Box<dyn Sound>, {} channel, {} samples",
                    self.controller.channel_count(),
                    self.controller.sample_rate()
                ),
            )
            .finish()
    }
}

impl AudioPlayer {
    pub fn new(volume: f32, speed: f32, pan: f32, file: String) -> Self {
        let frames = sounds::open_file(file).unwrap()
            .pausable()
            .with_adjustable_pan_of(pan)
            .with_adjustable_volume_of(volume)
            .with_adjustable_speed_of(speed)
            .controllable().0;

        Self {
            controller: frames,
        }
    }

    pub fn adjust_volume(&mut self, volume: f32) {
        self.controller.set_volume(volume);
    }

    pub fn adjust_speed(&mut self, speed: f32) {
        self.controller.set_speed(speed);
    }

    pub fn play(self, manager: &mut manager::Manager) {
        manager.play(Box::new(self.controller));
    }

    pub fn adjust_pan(&mut self, pan: f32) {
        self.controller.set_pan(pan);
    }
}
