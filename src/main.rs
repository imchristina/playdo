mod bus;
mod cpu;
mod ram;
mod rom;
mod interrupt;
mod gpu;

use bus::Bus;
use cpu::Cpu;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
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
            for _ in 0..100000 {
                self.cpu.step(&mut self.bus);
                self.bus.step();
            }
        }
        ui.ctx().request_repaint();

        self.cpu_panel(ui);

        self.gpu_panel(ui);
    }
}

impl DebuggerApp {
    fn cpu_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("cpu").show(ui, |ui| {
            ui.separator();
            ui.heading("CPU");
            ui.separator();
            ui.horizontal(|ui| {
                ui.monospace(format!("PC: {:08X}", self.cpu.pc));

                if ui.button("Step").clicked() {
                    self.running = false;
                    self.cpu.step(&mut self.bus);
                    self.bus.step();
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

            egui::Grid::new("cpu_ins").striped(true).num_columns(1).show(ui, |ui| {
                for i in -4..5 {
                    let pc = (i + (self.cpu.pc as i64)) as u32;
                    let data = self.bus.read_u32_debug(pc);
                    let line = format!("{:08X}: {:08X}", pc, data);
                    if pc != self.cpu.pc {
                        ui.monospace(line);
                    } else {
                        ui.monospace(line).highlight();
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.heading("Registers");
            ui.separator();
            egui::Grid::new("cpu_regs").striped(true).num_columns(4).show(ui, |ui| {
                for (i, reg) in self.cpu.regs.iter().enumerate() {
                    ui.monospace(format!("{:>2}: {:08X}", i, reg));
                    if (i + 1) % 4 == 0 { ui.end_row(); }
                }
                ui.monospace(format!("HI: {:08X}", self.cpu.hi));
                ui.monospace(format!("LO: {:08X}", self.cpu.lo));
            });

            ui.separator();
            ui.heading("COP0");
            ui.separator();
            egui::Grid::new("cop0_regs").striped(true).num_columns(4).show(ui, |ui| {
                for (i, reg) in self.cpu.cop0_regs.iter().enumerate() {
                    ui.monospace(format!("{:>2}: {:08X}", i, reg));
                    if (i + 1) % 4 == 0 { ui.end_row(); }
                }
            });

            ui.separator();
            ui.heading("Interrupt");
            ui.separator();
            ui.monospace(format!("I_STAT: {:032b}", self.bus.interrupt.i_stat));
            ui.monospace(format!("I_MASK: {:032b}", self.bus.interrupt.i_mask));
        });
    }

    fn gpu_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("gpu").show(ui, |ui| {
            ui.separator();
            ui.heading("GPU");
            ui.separator();

            ui.monospace(format!("GPUSTAT: {:032b}", self.bus.gpu.gpustat()));
            ui.monospace(format!("Texture page X base: {:032b}", self.bus.gpu.texture_page_x_base));
            ui.monospace(format!("Texture page Y base: {:032b}", self.bus.gpu.texture_page_y_base));
        });
    }
}
