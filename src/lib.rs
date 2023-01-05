use nih_plug::{
    buffer::Buffer,
    context::process::ProcessContext,
    midi::{MidiConfig, NoteEvent},
    params::Params,
    plugin::{AuxiliaryBuffers, Plugin, ProcessStatus},
};
use std::sync::Arc;

struct SustainPlugin {
    params: Arc<SustainPluginParams>,
}

impl Default for SustainPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(Default::default()),
        }
    }
}

#[derive(Default, Params)]
struct SustainPluginParams {
    is_sustaining: bool,
}

impl Plugin for SustainPlugin {
    const NAME: &'static str = "Sustain";
    const VENDOR: &'static str = "helixbass";
    const URL: &'static str = "https://github.com/helixbass/SustainPlugin";
    const EMAIL: &'static str = "julian@helixbass.net";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const DEFAULT_INPUT_CHANNELS: u32 = 0;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 0;

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOff { .. } => (),
                event => context.send_event(event),
            }
        }

        ProcessStatus::Normal
    }
}
