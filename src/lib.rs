use nih_plug::{
    buffer::Buffer,
    context::{gui::AsyncExecutor, process::ProcessContext},
    editor::Editor,
    midi::{MidiConfig, NoteEvent},
    params::Params,
    plugin::{AuxiliaryBuffers, Plugin, ProcessStatus, Vst3Plugin},
    prelude::{nih_export_vst3, BoolParam},
};
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Default)]
struct SustainPlugin {
    params: Arc<SustainPluginParams>,
    current_notes: HashMap<u8, NoteEvent>,
    sustained_notes: Option<HashMap<u8, NoteEvent>>,
}

#[derive(Params)]
struct SustainPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    // #[id = "is-sustaining"]
    // pub is_sustaining: BoolParam,
    is_sustaining: AtomicBool,
}

impl Default for SustainPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(300, 180),
            // is_sustaining: BoolParam::new("Is sustaining?", false),
            is_sustaining: Default::default(),
        }
    }
}

impl Plugin for SustainPlugin {
    const NAME: &'static str = "Sustain";
    const VENDOR: &'static str = "helixbass";
    const URL: &'static str = "https://github.com/helixbass/SustainPlugin";
    const EMAIL: &'static str = "julian@helixbass.net";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // const DEFAULT_INPUT_CHANNELS: u32 = 0;
    // const DEFAULT_OUTPUT_CHANNELS: u32 = 0;

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
        if self.params.is_sustaining.load(Ordering::SeqCst) && self.sustained_notes.is_none() {
            self.sustained_notes = Some(self.current_notes.clone());
        } else if !self.params.is_sustaining.load(Ordering::SeqCst)
            && self.sustained_notes.is_some()
        {
            let sustained_notes = self.sustained_notes.take().unwrap();
            for (sustained_note, sustained_note_event) in sustained_notes {
                match sustained_note_event {
                    NoteEvent::NoteOn {
                        timing,
                        voice_id,
                        channel,
                        note,
                        velocity,
                    } => {
                        context.send_event(NoteEvent::NoteOff {
                            timing,
                            voice_id,
                            channel,
                            note,
                            velocity,
                        });
                    }
                    _ => unreachable!(),
                }
                self.current_notes.remove(&sustained_note);
            }
        }

        while let Some(event) = context.next_event() {
            let mut dont_forward_event = false;
            match &event {
                NoteEvent::NoteOn { note, .. } => {
                    self.current_notes.insert(*note, event.clone());
                }
                NoteEvent::NoteOff { note, .. } => {
                    if matches!(
                        self.sustained_notes.as_ref(),
                        Some(sustained_notes) if sustained_notes.contains_key(note)
                    ) {
                        dont_forward_event = true;
                    } else {
                        self.current_notes.remove(note);
                    }
                }
                _ => (),
            }
            if !dont_forward_event {
                context.send_event(event);
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        create_egui_editor(self.params.editor_state.clone(), (), |_, _| {}, {
            let params = self.params.clone();
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    if !params.is_sustaining.load(Ordering::SeqCst)
                    /*.value()*/
                    {
                        if ui.add(egui::Button::new("Sustain")).clicked() {
                            params.is_sustaining.store(true, Ordering::SeqCst);
                        }
                    } else {
                        if ui.add(egui::Button::new("Stop sustaining")).clicked() {
                            params.is_sustaining.store(false, Ordering::SeqCst);
                        }
                    }
                });
            }
        })
    }
}

impl Vst3Plugin for SustainPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"SustainPluginzzz";
    const VST3_CATEGORIES: &'static str = "Instrument|Tools";
}

nih_export_vst3!(SustainPlugin);
