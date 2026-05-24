use leptos::html::Audio;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct AudioMeter {
    pub audio_ref: NodeRef<Audio>,
    pub current: RwSignal<f64>,
    pub duration: RwSignal<f64>,
    pub paused: RwSignal<bool>,
    pub volume: RwSignal<f64>,
}

impl AudioMeter {
    pub fn expect() -> Self {
        expect_context::<Self>()
    }
}
