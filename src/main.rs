use std::{collections::HashSet, path::PathBuf};

use clap::Parser;

mod geom;
mod lumatone;
mod midi;

use geom::TOTAL_SIZE;
use lumatone::Layout;
use midi::MidiState;

pub struct Visuals {
    pub outline_size: f32,
    pub darken_unpressed: f32,
    pub lighten_pressed: f32,
}

#[derive(clap::Parser, Debug)]
struct CliArgs {
    file: PathBuf,
}

fn main() -> eyre::Result<()> {
    let args = CliArgs::parse();

    let layout = Layout::load_from_file(&args.file)?;

    let mut midi = MidiState::default();

    let mut pressed_keys = HashSet::new();

    let mut visuals = Visuals {
        outline_size: 0.1,
        darken_unpressed: 0.6,
        lighten_pressed: 0.1,
    };

    eframe::run_simple_native(
        "Lumatone Visualization",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.columns(2, |uis| {
                    show_midi_ui(&mut uis[0], &mut midi);
                    show_visuals_ui(&mut uis[1], &mut visuals);
                });
            });

            if let MidiState::Connected(state) = &midi {
                while let Some(ev) = state.try_recv() {
                    if let midly::live::LiveEvent::Midi { channel, message } = ev {
                        match message {
                            midly::MidiMessage::NoteOff { key, vel: _ } => {
                                pressed_keys.remove(&(channel.as_int(), key.as_int()));
                            }
                            midly::MidiMessage::NoteOn { key, vel: _ } => {
                                pressed_keys.insert((channel.as_int(), key.as_int()));
                            }
                            _ => (),
                        }
                    }
                }
            } else {
                pressed_keys.clear();
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                let rect = ui.max_rect();
                let center = rect.center();
                let scale = (rect.size() / *TOTAL_SIZE).min_elem();

                let p = ui.painter();
                for board_index in 0..5 {
                    let board = &layout.boards[board_index];
                    for key_index in 0..56 {
                        let key = board.keys[key_index];

                        let points =
                            geom::hexagon_coordinates(board_index, key_index, visuals.outline_size)
                                .map(|v| center + v * scale)
                                .to_vec();

                        let [r, g, b] = key.color;
                        let key_color = egui::Color32::from_rgb(r, g, b);
                        let is_pressed = pressed_keys.contains(&(key.midi_chan, key.midi_note));
                        let fill_color = if is_pressed {
                            key_color
                                .blend(egui::Color32::WHITE.gamma_multiply(visuals.lighten_pressed))
                        } else {
                            key_color.gamma_multiply(1.0 - visuals.darken_unpressed)
                        };

                        let stroke = (
                            visuals.outline_size * scale,
                            if is_pressed {
                                ui.visuals().strong_text_color()
                            } else {
                                ui.visuals()
                                    .strong_text_color()
                                    .gamma_multiply(1.0 - visuals.darken_unpressed)
                            },
                        );

                        p.add(egui::epaint::PathShape::convex_polygon(
                            points, fill_color, stroke,
                        ));
                    }
                }
            });
        },
    )
    .expect("eframe error");

    Ok(())
}

fn show_midi_ui(ui: &mut egui::Ui, midi: &mut MidiState) {
    ui.group(|ui| {
        ui.strong("MIDI");
        match std::mem::take(midi) {
            MidiState::Uninit(error) => {
                let error_msg = error.as_ref().map(|e| e.to_string()).unwrap_or_default();
                if ui.button("Initialize").clicked() {
                    midi.init();
                } else {
                    ui.colored_label(ui.visuals().error_fg_color, error_msg);
                }
                *midi = MidiState::Uninit(error);
            }
            MidiState::Ready {
                state,
                mut input_port,
                mut output_port,
            } => {
                if ui.button("Uninitialize").clicked() {
                    midi.uninit();
                } else {
                    ui.label("Input port");
                    ui.horizontal(|ui| {
                        for (port, port_name) in state.input_ports() {
                            ui.selectable_value(&mut input_port, Some(port), port_name);
                        }
                    });
                    ui.label("Output port");
                    ui.horizontal(|ui| {
                        for (port, port_name) in state.output_ports() {
                            ui.selectable_value(&mut output_port, Some(port), port_name);
                        }
                    });
                    ui.add_enabled_ui(input_port.is_some() && output_port.is_some(), |ui| {
                        if ui.button("Connect").clicked() {
                            *midi = match state.connect(input_port.unwrap(), output_port.unwrap()) {
                                Ok(connected_state) => MidiState::Connected(connected_state),
                                Err(e) => e.into(),
                            };
                        } else {
                            *midi = MidiState::Ready {
                                state,
                                input_port,
                                output_port,
                            };
                        }
                    });
                }
            }
            MidiState::Connected(state) => {
                ui.label("Connected");
                ui.horizontal(|ui| {
                    ui.label("Input port");
                    ui.add_enabled_ui(false, |ui| ui.selectable_label(true, &state.input_port().1));
                });
                ui.horizontal(|ui| {
                    ui.label("Output port");
                    ui.add_enabled_ui(false, |ui| {
                        ui.selectable_label(true, &state.output_port().1)
                    });
                });
                if ui.button("Disconnect").clicked() {
                    *midi = MidiState::Ready {
                        state: state.disconnect(),
                        input_port: None,
                        output_port: None,
                    };
                } else {
                    *midi = MidiState::Connected(state);
                }
            }
        }
    });
}

fn show_visuals_ui(ui: &mut egui::Ui, visuals: &mut Visuals) {
    ui.group(|ui| {
        ui.strong("Visuals");
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut visuals.outline_size, 0.0..=0.5).fixed_decimals(2));
            ui.label("Outline");
        });
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut visuals.darken_unpressed, 0.0..=1.0).fixed_decimals(2));
            ui.label("Darken unpressed");
        });
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut visuals.lighten_pressed, 0.0..=1.0).fixed_decimals(2));
            ui.label("Lighten pressed");
        });
    });
}
