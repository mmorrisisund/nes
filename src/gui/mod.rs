mod debug_tools;
mod ppu_viewer;

use debug_tools::DebugTools;
use eframe::egui::{menu, Context, TopBottomPanel, Ui, ViewportCommand};

pub struct Gui {
    debug_tools: DebugTools,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            debug_tools: DebugTools::default(),
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        TopBottomPanel::top("main_menu_bar").show(ctx, |ui| self.menubar(ctx, ui));

        self.debug_tools.show(ctx);
    }

    fn menubar(&mut self, ctx: &Context, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load ROM").clicked() {
                    ui.close_menu();
                }
                if ui.button("Show debug tools").clicked() {
                    self.debug_tools.set_open(true);
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            })
        });
    }
}

impl Default for Gui {
    fn default() -> Self {
        Self::new()
    }
}
