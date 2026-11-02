use crate::cartridge::CartridgeMapper;
use crate::cpu::Cpu;
use crate::mmu::Mmu;

pub struct Emulator {
    cpu: Cpu,
    mmu: Mmu,
}

impl Emulator {
    pub fn new(cartridge: Box<dyn CartridgeMapper>) -> Self {
        Self {
            cpu: Cpu::default(),
            mmu: Mmu::new(cartridge),
        }
    }

    pub fn run(&mut self)
    {
        loop {
            self.cpu.handle_interrupts(&mut self.mmu);

            let instruction = self.cpu.fetch_next_instruction(&mut self.mmu);

            self.cpu.handle_instruction(&mut self.mmu, instruction);
        }
    }
}
