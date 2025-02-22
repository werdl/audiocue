use std::sync::mpsc::Receiver;

use awedio::{
    manager, sounds::{
        self,
        wrappers::{AdjustableSpeed, AdjustableVolume, CompletionNotifier, Controllable, Pausable, SetPaused, SetSpeed, SetVolume, Wrapper},
        Silence,
    }, Sound
};

use crate::pan::{Pan, Panned, SetPan};

pub struct AudioPlayer {
    controller: Controllable<AdjustableSpeed<AdjustableVolume<CompletionNotifier<Panned<Pausable<Box<dyn Sound>>>>>>>,

    pub completion_notifier: Receiver<()>,
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
    pub fn new(volume: f32, speed: f32, pan_lr: f32, pan_fb: f32, file: String) -> Self {
        let (frames, completion_notifier) = sounds::open_file(file).unwrap()
            .pausable()
            .with_adjustable_pan_of(pan_lr, pan_fb)
            .with_completion_notifier();

        let frames = frames
            .with_adjustable_volume_of(volume)
            .with_adjustable_speed_of(speed)
            .controllable().0;

        Self {
            controller: frames,
            completion_notifier
        }
    }

    pub fn adjust_volume(&mut self, volume: f32) {
        self.controller.set_volume(volume);
    }

    pub fn adjust_speed(&mut self, speed: f32) {
        self.controller.set_speed(speed);
    }

    pub fn adjust_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.controller.inner_mut().inner_mut().set_pan(pan_lr, pan_fb);
    }

    pub fn play(self, manager: &mut manager::Manager) {
        manager.play(Box::new(self.controller));
    }

    pub fn play_blocking(self, manager: &mut manager::Manager) {
        manager.play(Box::new(self.controller));

        loop {
            if self.completion_notifier.try_recv().is_ok() {
                break;
            }
        }
    }
}
