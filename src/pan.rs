use awedio::sounds::wrappers::{AdjustableSpeed, AdjustableVolume, AsyncCompletionNotifier, CompletionNotifier, Controllable, Controller, FinishAfter, Pausable, SetPaused, SetSpeed, SetVolume, Wrapper};
use awedio::sounds::MemorySound;
use awedio::{NextSample, Sound, Error};
use std::sync::mpsc::Receiver;
use std::time::Duration;

enum PanDir {
    Left,
    Right,

    /// 3 channel audio, for instance
    Center,
}

fn pan_dir(channel_count: u16, current_channel: u16) -> PanDir {
    match channel_count {
        1 => PanDir::Center,
        2 => match current_channel {
            0 => PanDir::Left,
            1 => PanDir::Right,
            _ => unreachable!(),
        },
        3 => match current_channel {
            0 => PanDir::Left,
            1 => PanDir::Center,
            2 => PanDir::Right,
            _ => unreachable!(),
        },
        4 => match current_channel {
            0 => PanDir::Left, // front left
            1 => PanDir::Right, // front right
            2 => PanDir::Left, // rear left
            3 => PanDir::Right, // rear right
            _ => unreachable!(),
        },
        _ => unimplemented!("Pan not implemented for {} channels", channel_count),
    }
}

pub struct Panned<S: Sound> {
    pub pan: f32,
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
            let pan = self.pan;
            let dir = pan_dir(self.channel_count(), self.current_channel);
            let adjusted = match dir {
                PanDir::Left => (s as f32 * (1.0 + pan)) as i16,
                PanDir::Right => (s as f32 * (1.0 - pan)) as i16,
                PanDir::Center => s,
            };
            NextSample::Sample(adjusted)
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
            pan: 0.0,
            sound: self,
            current_channel: 0,
        }
    }

    fn with_adjustable_pan_of(self, pan: f32) -> Panned<Self>
    where
        Self: Sized,
    {
        Panned {
            pan,
            sound: self,
            current_channel: 0,
        }
    }
}

pub trait SetPan where Self: Sound {
    fn set_pan(&mut self, pan: f32);
}

impl<S: Sound> Pan for S {}

impl<S: Sound> SetPan for Panned<S> {
    fn set_pan(&mut self, pan: f32) {
        self.pan = pan;
    }
}

pub struct AdjustablePan<S: Sound> {
    inner: S,
    pan_adjustment: f32,
}

impl<S> AdjustablePan<S>
where
    S: Sound,
{
    pub fn new(inner: S) -> Self {
        AdjustablePan {
            inner,
            pan_adjustment: 0.0,
        }
    }

    pub fn new_with_pan(inner: S, pan_adjustment: f32) -> Self {
        AdjustablePan {
            inner,
            pan_adjustment,
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
                let adjusted = (s as f32 * self.pan_adjustment) as i16;
                NextSample::Sample(adjusted)
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
    pub fn pan(&self) -> f32 {
        self.pan_adjustment
    }
}

impl<S> SetPan for AdjustablePan<S>
where
    S: Sound,
{
    fn set_pan(&mut self, new: f32) {
        self.pan_adjustment = new;
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
    fn set_pan(&mut self, pan: f32) {
        self.inner_mut().set_pan(pan)
    }
}

impl<S> SetPan for AdjustableSpeed<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan: f32) {
        self.inner_mut().set_pan(pan)
    }
}

impl<S> SetPan for Pausable<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan: f32) {
        self.inner_mut().set_pan(pan)
    }
}

impl<S> SetPan for FinishAfter<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan: f32) {
        self.inner_mut().set_pan(pan)
    }
}

impl<S> SetPan for Controllable<S> 
where
    S: Sound + SetPan,
{
    fn set_pan(&mut self, pan: f32) {
        self.inner_mut().set_pan(pan)
    }
}