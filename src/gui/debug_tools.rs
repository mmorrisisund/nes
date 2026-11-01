use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use eframe::egui::{CentralPanel, Context, ScrollArea, Ui, Vec2, ViewportBuilder, ViewportId};
use parking_lot::Mutex;

use super::ppu_viewer::PpuViewer;

#[derive(Debug)]
pub struct DebugTools {
    id: ViewportId,
    open: Arc<AtomicBool>,
    state: Arc<Mutex<State>>,
}

#[derive(Debug, Default)]
pub struct State {
    tab: Tab,
    ppu: PpuViewer,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Cpu,
    Ppu,
}

impl DebugTools {
    const TITLE: &'static str = "🕷 Debug Tools";

    pub fn show(&mut self, ctx: &Context) {
        if !self.open() {
            return;
        }

        let open = self.open.clone();
        let state = self.state.clone();
        let viewport = ViewportBuilder::default()
            .with_title(Self::TITLE)
            .with_inner_size(Vec2::new(1024.0, 768.0));

        ctx.show_viewport_deferred(self.id, viewport, move |ctx, _class| {
            CentralPanel::default().show(ctx, |ui| state.lock().ui(ui));

            if ctx.input(|i| i.viewport().close_requested()) {
                open.store(false, Ordering::Relaxed);
            }
        });
    }

    pub fn open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    pub fn set_open(&self, open: bool) {
        self.open.store(open, Ordering::Relaxed);
    }
}

impl Default for DebugTools {
    fn default() -> Self {
        Self {
            id: ViewportId::from_hash_of(Self::TITLE),
            open: Default::default(),
            state: Arc::new(Mutex::new(State::default())),
        }
    }
}

impl State {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Cpu, "CPU");
            ui.selectable_value(&mut self.tab, Tab::Ppu, "PPU");
        });

        ui.separator();

        ScrollArea::both().show(ui, |ui| match self.tab {
            Tab::Cpu => self.cpu_tab(ui),
            Tab::Ppu => self.ppu_tab(ui),
        });
    }

    fn cpu_tab(&mut self, ui: &mut Ui) {
        ui.label("CPU Tab");
    }

    fn ppu_tab(&mut self, ui: &mut Ui) {
        self.ppu.show(ui);
    }
}
