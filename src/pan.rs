use awedio::sounds::wrappers::{AdjustableSpeed, AdjustableVolume, AsyncCompletionNotifier, CompletionNotifier, Controllable, Controller, FinishAfter, Pausable, SetPaused, SetSpeed, SetVolume, Wrapper};
use awedio::sounds::MemorySound;
use awedio::{NextSample, Sound, Error};
use std::sync::mpsc::Receiver;
use std::time::Duration;

enum PanDir {
    Left,
    Right,
    Center,
}

enum TiltDir {
    Front,
    Rear,
    Center,
}

fn pan_dir(channel_count: u16, current_channel: u16) -> (PanDir, TiltDir) {
    match channel_count {
        1 => (PanDir::Center, TiltDir::Center),
        2 => match current_channel {
            0 => (PanDir::Left, TiltDir::Center),
            1 => (PanDir::Right, TiltDir::Center),
            _ => unreachable!(),
        },
        3 => match current_channel {
            0 => (PanDir::Left, TiltDir::Center),
            1 => (PanDir::Right, TiltDir::Center),
            2 => (PanDir::Center, TiltDir::Center),
            _ => unreachable!(),
        },
        4 => match current_channel {
            0 => (PanDir::Left, TiltDir::Front),
            1 => (PanDir::Right, TiltDir::Front),
            2 => (PanDir::Left, TiltDir::Rear),
            3 => (PanDir::Right, TiltDir::Rear),
            _ => unreachable!(),
        },
        _ => unimplemented!("Pan not implemented for {} channels", channel_count),
    }
}

pub struct Panned<S: Sound> {
    pub pan_lr: f32,
    pub pan_fb: f32,
    pub sound: S,
    pub current_channel: u16,
}

impl<S: Sound> Sound for Panned<S> {
    fn channel_count(&self) -> u16 {
        self.sound.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.sound.sample_rate()
    }

    fn next_sample(&mut self) -> Result<NextSample, Error> {
        let next = self.sound.next_sample()?;

        // increase the channel count if the current channel is less than the channel count, else reset the channel count to 0
        self.current_channel = if self.current_channel < self.channel_count() - 1 {
            self.current_channel + 1
        } else {
            0
        };

        Ok(match next {
            NextSample::Sample(s) => {
            let pan_lr = self.pan_lr;
            let pan_fb = self.pan_fb;
            let (pan_dir, tilt_dir) = pan_dir(self.channel_count(), self.current_channel);
            // -1.0 is left, 1.0 is right
            // -1.0 is front, 1.0 is rear

            let adjusted_lr = match pan_dir {
                PanDir::Left => s as f32 * (1.0 + pan_lr),
                PanDir::Right => s as f32 * (1.0 - pan_lr),
                PanDir::Center => s as f32,
            };

            let adjusted_fb = match tilt_dir {
                TiltDir::Front => adjusted_lr as f32 * (1.0 + pan_fb),
                TiltDir::Rear => adjusted_lr as f32 * (1.0 - pan_fb),
                TiltDir::Center => adjusted_lr as f32,
            };
            

            NextSample::Sample(adjusted_fb as i16)
            }
            NextSample::MetadataChanged
            | NextSample::Paused
            | NextSample::Finished => next,
        })

    }

    fn on_start_of_batch(&mut self) {
        self.sound.on_start_of_batch()
    }
}

pub trait Pan where Self: Sound {
    fn with_adjustable_pan(self) -> Panned<Self>
    where
        Self: Sized,
    {
        Panned {
            pan_lr: 0.0,
            pan_fb: 0.0,
            sound: self,
            current_channel: 0,
        }
    }

    fn with_adjustable_pan_of(self, pan_left_right: f32, pan_front_back: f32) -> Panned<Self>
    where
        Self: Sized,
    {
        Panned {
            pan_lr: pan_left_right,
            pan_fb: pan_front_back,
            sound: self,
            current_channel: 0,
        }
    }
}

pub trait SetPan where Self: Sound {
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32);
}

impl<S: Sound> Pan for S {}

impl<S: Sound> SetPan for Panned<S> {
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.pan_lr = pan_lr;
        self.pan_fb = pan_fb;
    }
}

pub struct AdjustablePan<S: Sound> {
    inner: S,
    pan_lr: f32,
    pan_fb: f32,
}

impl<S> AdjustablePan<S>
where
    S: Sound,
{
    pub fn new(inner: S) -> Self {
        AdjustablePan {
            inner,
            pan_lr: 0.0,
            pan_fb: 0.0,
        }
    }

    pub fn new_with_pan(inner: S, pan_lr: f32, pan_fb: f32) -> Self {
        AdjustablePan {
            inner,
            pan_lr,
            pan_fb,
        }
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S> Sound for AdjustablePan<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn next_sample(&mut self) -> Result<NextSample, Error> {
        let next = self.inner.next_sample()?;
        Ok(match next {
            NextSample::Sample(s) => {
                let adjusted_lr = (s as f32 * self.pan_lr) as i16;
                let adjusted_fb = (adjusted_lr as f32 * self.pan_fb) as i16;
                NextSample::Sample(adjusted_fb)
            }
            NextSample::MetadataChanged
            | NextSample::Paused
            | NextSample::Finished => next,
        })
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S> AdjustablePan<S>
where
    S: Sound,
{
    pub fn pan_lr(&self) -> f32 {
        self.pan_lr
    }

    pub fn pan_fb(&self) -> f32 {
        self.pan_fb
    }
}

impl<S> SetPan for AdjustablePan<S>
where
    S: Sound,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.pan_lr = pan_lr;
        self.pan_fb = pan_fb;
    }
}

impl<S> SetPaused for AdjustablePan<S>
where
    S: Sound + SetPaused,
{
    fn set_paused(&mut self, paused: bool) {
        self.inner.set_paused(paused)
    }
}

impl<S> SetSpeed for AdjustablePan<S>
where
    S: Sound + SetSpeed,
{
    fn set_speed(&mut self, multiplier: f32) {
        self.inner.set_speed(multiplier)
    }
}

// now implement SetPan for speed, volume and paused
impl<S> SetVolume for AdjustablePan<S>
where
    S: Sound + SetVolume,
{
    fn set_volume(&mut self, volume: f32) {
        self.inner.set_volume(volume)
    }
}

impl<S> SetPan for AdjustableVolume<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.inner_mut().set_pan(pan_lr, pan_fb)
    }
}

impl<S> SetPan for AdjustableSpeed<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.inner_mut().set_pan(pan_lr, pan_fb)
    }
}
impl<S> SetPan for Pausable<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.inner_mut().set_pan(pan_lr, pan_fb)
    }
}

impl<S> SetPan for FinishAfter<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.inner_mut().set_pan(pan_lr, pan_fb)
    }
}

impl<S> SetPan for Controllable<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.inner_mut().set_pan(pan_lr, pan_fb)
    }
}

impl<S> SetPan for CompletionNotifier<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan_lr: f32, pan_fb: f32) {
        self.inner_mut().set_pan(pan_lr, pan_fb)
    }
}