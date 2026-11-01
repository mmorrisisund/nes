use eframe::egui::{
    CentralPanel, ColorImage, Image, ScrollArea, SidePanel, TextureFilter, TextureHandle,
    TextureOptions, Ui,
};

use crate::ppu::{HEIGHT, PIXEL_COUNT, WIDTH};

#[derive(Default)]
pub struct PpuViewer {
    tab: Tab,
    nametable_texture: Option<TextureHandle>,
}

#[derive(Debug, Default)]
enum Tab {
    #[default]
    Nametables,
    PatternTables,
    Oam,
    Palette,
}

impl PpuViewer {
    pub fn show(&mut self, ui: &mut Ui) {
        if self.nametable_texture.is_none() {
            self.nametable_texture = Some(ui.ctx().load_texture(
                "ppu_viewer_nametable",
                ColorImage::from_rgb([HEIGHT, WIDTH], &[0; PIXEL_COUNT * 3]),
                TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Nearest,
                    ..Default::default()
                },
            ));
        }

        SidePanel::right("nametable_info").show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(12.0);
                ui.heading("Nametable Info");
                ui.separator();
            });
        });

        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.add(
                    Image::from_texture(self.nametable_texture.as_ref().unwrap())
                        .fit_to_exact_size([3.0 * HEIGHT as f32, 3.0 * WIDTH as f32].into()),
                );
            });
        });
    }
}

impl std::fmt::Debug for PpuViewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PpuViewer")
            .field("tab", &self.tab)
            .field(
                "nametable_texture",
                &self.nametable_texture.as_ref().unwrap().name(),
            )
            .finish()
    }
}
