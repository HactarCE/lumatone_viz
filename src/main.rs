use std::path::PathBuf;

use clap::Parser;

mod geom;
mod lumatone;

use geom::TOTAL_SIZE;
use lumatone::Layout;

#[derive(clap::Parser, Debug)]
struct CliArgs {
    file: PathBuf,
}

fn main() -> eyre::Result<()> {
    let args = CliArgs::parse();

    let layout = Layout::load_from_file(&args.file)?;

    eframe::run_simple_native(
        "Lumatone Visualization",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Hello, world!");

                let rect = ui.max_rect();
                let center = rect.center();
                let scale = (rect.size() / *TOTAL_SIZE).min_elem();

                let p = ui.painter();
                for board in 0..5 {
                    for index in 0..56 {
                        let [r, g, b] = layout.boards[board].notes[index].color;
                        p.add(egui::epaint::PathShape::convex_polygon(
                            geom::hexagon_coordinates(board, index)
                                .map(|v| center + v * scale)
                                .to_vec(),
                            egui::Color32::from_rgb(r, g, b),
                            (2.0, ui.visuals().strong_text_color()),
                        ));
                    }
                }
            });
        },
    )
    .expect("eframe error");

    Ok(())
}
