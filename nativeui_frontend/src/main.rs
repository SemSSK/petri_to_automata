use std::fs;

use backend::{graph_gen::compile_to_output, output_generators, petri_parser::parser::PetriNet};
use eframe::glow::Texture;
use egui::{Color32, ColorImage, TextEdit, TextureHandle, TextureId, Vec2};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    code: String,
    image: Option<TextureHandle>,
}

impl MyApp {
    fn build(&mut self, ui: &egui::Ui) {
        let output =
            compile_to_output(PetriNet::new(&self.code).unwrap().generate_input()).unwrap();
        let buffer = img.to_rgba8().into_vec();
        let size = [img.width() as usize, img.height() as usize];
        let pixels = buffer
            .chunks_exact(4)
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect::<Vec<_>>();
        let image = ColorImage { size, pixels };
        self.image = Some(
            ui.ctx()
                .load_texture("build result", image, Default::default()),
        );
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            code: Default::default(),
            image: Default::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("build 🔨").clicked() {
                            self.build(ui);
                        }
                    });
                    ui.text_edit_multiline(&mut self.code);
                });
                columns[1].vertical(|ui| match &self.image {
                    Some(image) => {
                        ui.image(image, image.size_vec2());
                    }
                    None => {
                        ui.label("nothing to build");
                    }
                });
            });
        });
    }
}
