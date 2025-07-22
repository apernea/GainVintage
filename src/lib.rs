use nih_plug::{prelude::*, util::db_to_gain};
use nih_plug_egui::EguiState;
use std::sync::{Arc, atomic::Ordering};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Tube,
    Tape,
}

impl Enum for Mode {
    fn variants() -> &'static [&'static str] {
        &["Tube", "Tape"]
    }

    fn ids() -> &'static [&'static str] {
        &["tube", "tape"]
    }

    fn to_index(self) -> usize {
        match self {
            Mode::Tube => 0,
            Mode::Tape => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Mode::Tube,
            1 => Mode::Tape,
            _ => Mode::Tube, // fallback
        }
    }
}

pub struct GainVintage{
    params: Arc<PluginParams>,
    peak_meter_decay_factor: f32,
    peak_meter: Arc<AtomicF32>,
}

pub struct PluginParams{
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "mode"]
    pub mode: EnumParam<Mode>,

    #[id = "gain"]
    pub gain:FloatParam,

    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "input_trim"]
    pub input_trim: FloatParam,

    #[id = "output_trim"]
    pub output_trim: FloatParam,

    #[id = "mute"]
    pub mute: BoolParam,
}

impl Default for GainVintage{
    fn default() -> Self {
        Self{
            params: Arc::new(PluginParams::default()),
            peak_meter_decay_factor: 0.9996,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

impl Default for PluginParams{
    fn default() -> Self {
        Self{
            editor_state: EguiState::from_size(400, 300),
            mode: EnumParam::new("Mode", Mode::Tube),
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -10.0,
                    max: 10.0,
                },
            )
                .with_step_size(0.1)
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" dB"),
            drive: FloatParam::new("Drive", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            input_trim: FloatParam::new("Input Trim", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 }),
            output_trim: FloatParam::new("Output Trim", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 }),
            mute: BoolParam::new("Mute", false),
        }
    }
}

impl Plugin for GainVintage{
    const NAME: &'static str = "GainVintage";
    const VENDOR: &'static str = "AlexPernea";
    const URL: &'static str = "https://www.beatstars.com/alexpernea";
    const EMAIL: &'static str = "axp6745@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];
    
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.peak_meter.clone())
    }
    fn process(&mut self, buffer: &mut Buffer, aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        todo!()
    }
}