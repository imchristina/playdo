mod bus;
mod cpu;
mod rom;

use bus::Bus;
use cpu::Cpu;

fn main() {
    let mut bus = Bus::new("SCPH1001.BIN");
    let mut cpu = Cpu::new();

    loop {
        cpu.step(&mut bus);
    }
}
