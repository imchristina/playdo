mod bus;
mod cpu;
mod ram;
mod rom;

use bus::Bus;
use cpu::Cpu;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "Playdo",
        options,
        Box::new(|_cc| {
            Ok(Box::<DebuggerApp>::default())
        }),
    )
}

struct DebuggerApp {
    bus: Bus,
    cpu: Cpu,

    running: bool,
}

impl Default for DebuggerApp {
    fn default() -> Self {
        Self {
            bus: Bus::new("SCPH1001.BIN"),
            cpu: Cpu::new(),

            running: false,
        }
    }
}

impl eframe::App for DebuggerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.running {
            for _ in 0..10000 {
                self.cpu.step(&mut self.bus);
            }
        }
        ui.ctx().request_repaint();

        self.cpu_window(ui);
        self.cop0_window(ui);
    }
}

impl DebuggerApp {
    fn cpu_window(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("CPU").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.monospace(format!("PC '{:#X}'", self.cpu.pc));

                if ui.button("Step").clicked() {
                    self.running = false;
                    self.cpu.step(&mut self.bus);
                }

                if self.running {
                    if ui.button("Stop").clicked() {
                        self.running = false;
                    }
                } else {
                    if ui.button("Run").clicked() {
                        self.running = true;
                    }
                }
            });

            egui::Grid::new("cpu_regs").striped(true).num_columns(4).show(ui, |ui| {
                for (i, reg) in self.cpu.regs.iter().enumerate() {
                    ui.monospace(format!("{:>2}: {:08X}", i, reg));
                    if (i + 1) % 4 == 0 { ui.end_row(); }
                }
                ui.monospace(format!("HI: {:08X}", self.cpu.hi));
                ui.monospace(format!("LO: {:08X}", self.cpu.lo));
            });
        });
    }

    fn cop0_window(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("COP0").show(ui, |ui| {
            egui::Grid::new("cop0_regs").striped(true).num_columns(4).show(ui, |ui| {
                for (i, reg) in self.cpu.cop0_regs.iter().enumerate() {
                    ui.monospace(format!("{:>2}: {:08X}", i, reg));
                    if (i + 1) % 4 == 0 { ui.end_row(); }
                }
            });
        });
    }
}
