mod cartridge;

fn main() {
    let args = std::env::args().skip(1).collect::<Box<[_]>>();

    let rom_path = if args.len() == 1 {
        args[0].to_owned()
    } else {
        panic!("usage")
    };

    drop(args);

    let cartridge = cartridge::load_cartridge_path(rom_path).unwrap();

    _ = cartridge;
}
