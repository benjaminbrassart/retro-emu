mod bus;
mod cartridge;
mod cpu;
mod display;
mod emulator;
mod mmu;
mod registers;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Box<[_]>>();

    // Check args
    if args.len() < 1 || args.len() > 2 {
        return Err("wrong number of args".into());
    }

    let cartridge = crate::cartridge::load_cartridge_path(&args[0])?;

    drop(args);

    let index: i32 = 10;
    let (tx, rx) = mpsc::channel::<i32>();
    let pause = Arc::new(AtomicBool::new(false));
    let paused_emu = Arc::clone(&pause);

    thread::spawn(move || {
        let mut emulator = crate::emulator::Emulator::new(cartridge);

        emulator.run();
    });

    thread::spawn(move || {
        let mut count: i32 = 0;
        while count < index {
            if paused_emu.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            count += 1;
            println!("count: {}", count);
            if tx.send(count).is_err() {
                break;
            } // send value of count
            thread::sleep(Duration::from_millis(500));
        }
    });

    display::create_window(rx, pause);

    Ok(())
}
