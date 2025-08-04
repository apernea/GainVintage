mod editor;

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

    fn ids() -> Option<&'static [&'static str]> {
        Some(&["tube", "tape"])
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
            _ => Mode::Tube,
        }
    }
}

pub struct GainVintage{
    params: Arc<PluginParams>,
    peak_meter_decay_factor: f32,
    peak_meter: Arc<AtomicF32>,

    params_wrapper: Option<Arc<std::sync::Mutex<Option<Arc<PluginParams>>>>>,
    peak_meter_wrapper: Option<Arc<std::sync::Mutex<Option<Arc<AtomicF32>>>>>,
}

#[derive(Params)]
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
}

impl Default for GainVintage {
    fn default() -> Self {
        let params = Arc::new(PluginParams::default());
        let peak_meter = Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB));

        Self {
            params: params.clone(),
            peak_meter_decay_factor: 0.9996,
            peak_meter: peak_meter.clone(),

            params_wrapper: Some(Arc::new(std::sync::Mutex::new(Some(params)))),
            peak_meter_wrapper: Some(Arc::new(std::sync::Mutex::new(Some(peak_meter)))),
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        if let (Some(params_wrapper), Some(peak_meter_wrapper)) =
            (self.params_wrapper.clone(), self.peak_meter_wrapper.clone())
        {
            editor::create_wrapped(params_wrapper, peak_meter_wrapper)
        } else {
            None
        }
    }

    fn initialize(&mut self,
                  _audio_io_layout: &AudioIOLayout,
                  _buffer_config: &BufferConfig,
                  _context: &mut impl InitContext<Self>) -> bool {
        true
    }

    fn reset(&mut self) {}
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        use nih_plug::util::db_to_gain;
        use std::sync::atomic::Ordering;

        fn soft_clip(x: f32) -> f32 {
            const CLIP_THRESHOLD: f32 = 0.95;
            if x > CLIP_THRESHOLD {
                CLIP_THRESHOLD + (x - CLIP_THRESHOLD) / (1.0 + ((x - CLIP_THRESHOLD) / (1.0 - CLIP_THRESHOLD)).powi(2))
            } else if x < -CLIP_THRESHOLD {
                -CLIP_THRESHOLD + (x + CLIP_THRESHOLD) / (1.0 + ((-x - CLIP_THRESHOLD) / (1.0 - CLIP_THRESHOLD)).powi(2))
            } else {
                x
            }
        }

        for channel_samples in buffer.iter_samples() {
            let mut block_peak = 0.0f32;

            // --- 1. Fetch parameter values ---
            let input_trim_db = self.params.input_trim.smoothed.next();
            let output_trim_db = self.params.output_trim.smoothed.next();
            let gain_db = self.params.gain.smoothed.next();
            let drive = self.params.drive.value();
            let mode = self.params.mode.value();

            // --- 2. Convert to linear gains ---
            let input_gain = db_to_gain(input_trim_db);
            let output_gain = db_to_gain(output_trim_db);
            let gain = db_to_gain(gain_db);

            // --- 3. Process each sample in this channel ---
            for sample in channel_samples {
                let mut x = *sample;

                // Apply input trim
                x *= input_gain;

                // Store dry version for blending
                let dry = x;

                // Apply saturation
                x = match mode {
                    Mode::Tube => {
                        // More noticeable Tube distortion
                        let shaped = (x * (1.5 + drive)).tanh() + 0.15 * x.powi(3);
                        shaped
                    }
                    Mode::Tape => {
                        // Tape-style tanh + output volume control
                        let shaped = (x * drive * 3.5).tanh() * 0.8;
                        shaped
                    }
                };

                // Blend dry/wet using drive parameter
                x = dry * (1.0 - drive) + x * drive;

                // Apply main gain
                x *= gain;

                // Apply output trim
                x *= output_gain;

                // Apply final soft clipping
                x = soft_clip(x);

                // Store back into buffer
                *sample = x;

                // Track peak
                block_peak = block_peak.max(x.abs());
            }

            // --- 4. Update Peak Meter (if editor is open) ---
            if self.params.editor_state.is_open() {
                let current = self.peak_meter.load(Ordering::Relaxed);

                let new_peak = if block_peak > current {
                    block_peak // fast attack
                } else {
                    current * self.peak_meter_decay_factor // slow decay
                };

                let clamped = if new_peak < util::MINUS_INFINITY_GAIN {
                    0.0
                } else {
                    new_peak
                };

                self.peak_meter.store(clamped, Ordering::Relaxed);
            }
        }

        ProcessStatus::Normal
    }
}

impl Drop for GainVintage {
    fn drop(&mut self) {
        if let Some(ref wrapper) = self.params_wrapper {
            if let Ok(mut lock) = wrapper.lock() {
                *lock = None;
            }
        }
        if let Some(ref wrapper) = self.peak_meter_wrapper {
            if let Ok(mut lock) = wrapper.lock() {
                *lock = None;
            }
        }
    }
}

impl ClapPlugin for GainVintage{
    const CLAP_ID: &'static str = "com.alexpernea.gainvintage";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Vintage gain plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for GainVintage {
    const VST3_CLASS_ID: [u8; 16] = [
        0xf4, 0xa1, 0x61, 0x26,
        0x6c, 0x8e,
        0x4c, 0xf0,
        0x99, 0x73,
        0x5f, 0x6a, 0xeb, 0xab, 0xc1, 0x19,
    ];
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(GainVintage);
nih_export_vst3!(GainVintage);