mod bus;
mod cpu;
mod mmu;

fn main() {
    let _ = cpu::Cpu::default();

    println!("Hello, world!");
}
