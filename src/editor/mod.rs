use nih_plug::editor::Editor;
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{ComboBox, Slider, Vec2},
    resizable_window::ResizableWindow,
};
use std::sync::{Arc, Weak, Mutex};

use crate::{Mode, PluginParams};

mod level_meter;
mod toggle;

use level_meter::PeakMeter;

pub(crate) fn create(
    params: Arc<PluginParams>,
    peak_meter: Arc<AtomicF32>,
) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();

    let params_wrapper = Arc::new(Mutex::new(Some(params)));
    let peak_meter_wrapper = Arc::new(Mutex::new(Some(peak_meter)));

    let weak_params = Arc::downgrade(&params_wrapper);
    let weak_peak_meter = Arc::downgrade(&peak_meter_wrapper);

    create_egui_editor(
        egui_state.clone(),
        (),
        |_, _| {},
        move |ctx, setter, _state| {
            if let (Some(params_arc), Some(peak_meter_arc)) = (weak_params.upgrade(), weak_peak_meter.upgrade()) {
                let params_guard = params_arc.lock().unwrap();
                let peak_meter_guard = peak_meter_arc.lock().unwrap();

                if let (Some(params), Some(peak_meter)) = (&*params_guard, &*peak_meter_guard) {
                    ResizableWindow::new("GainVintage")
                        .min_size(Vec2::new(800.0, 600.0))
                        .show(ctx, egui_state.as_ref(), |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("GainVintage by Alex Pernea");
                                ui.add_space(10.0);

                                let peak_db = util::gain_to_db(
                                    peak_meter.load(std::sync::atomic::Ordering::Relaxed),
                                );

                                ui.label(format!("{:.1} dB", peak_db));
                                ui.add(
                                    PeakMeter::new(-60.0..=0.0, peak_db)
                                        .with_size(Vec2::new(24.0, 180.0))
                                        .show_label(false),
                                );

                                ui.add_space(30.0);
                            });

                            ui.vertical(|ui| {
                                ui.label("Gain");
                                ui.add(
                                    Slider::from_get_set(-10.0..=10.0, |v| match v {
                                        Some(new) => {
                                            setter.begin_set_parameter(&params.gain);
                                            setter.set_parameter(&params.gain, new as f32);
                                            setter.end_set_parameter(&params.gain);
                                            new
                                        }
                                        None => params.gain.value() as f64,
                                    })
                                        .suffix(" dB"),
                                );

                                ui.label("Input Trim");
                                ui.add(
                                    Slider::from_get_set(-12.0..=12.0, |v| match v {
                                        Some(new) => {
                                            setter.begin_set_parameter(&params.input_trim);
                                            setter.set_parameter(&params.input_trim, new as f32);
                                            setter.end_set_parameter(&params.input_trim);
                                            new
                                        }
                                        None => params.input_trim.value() as f64,
                                    })
                                        .suffix(" dB"),
                                );

                                ui.label("Output Trim");
                                ui.add(
                                    Slider::from_get_set(-12.0..=12.0, |v| match v {
                                        Some(new) => {
                                            setter.begin_set_parameter(&params.output_trim);
                                            setter.set_parameter(&params.output_trim, new as f32);
                                            setter.end_set_parameter(&params.output_trim);
                                            new
                                        }
                                        None => params.output_trim.value() as f64,
                                    })
                                        .suffix(" dB"),
                                );

                                ui.label("Drive");
                                ui.add(
                                    Slider::from_get_set(0.0..=1.0, |v| match v {
                                        Some(new) => {
                                            setter.begin_set_parameter(&params.drive);
                                            setter.set_parameter(&params.drive, new as f32);
                                            setter.end_set_parameter(&params.drive);
                                            new
                                        }
                                        None => params.drive.value() as f64,
                                    }),
                                );

                                ui.label("Mode");
                                let mut mode = params.mode.value();
                                ComboBox::from_id_source("mode")
                                    .selected_text(format!("{:?}", mode))
                                    .show_ui(ui, |ui| {
                                        for (i, &label) in Mode::variants().iter().enumerate() {
                                            ui.selectable_value(&mut mode, Mode::from_index(i), label);
                                        }
                                    });
                                if mode != params.mode.value() {
                                    setter.begin_set_parameter(&params.mode);
                                    setter.set_parameter(&params.mode, mode);
                                    setter.end_set_parameter(&params.mode);
                                }
                            });
                        });
                }
            }
        },
    )
}