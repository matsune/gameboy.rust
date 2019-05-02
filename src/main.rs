use std::fs::File;
use std::io::Read;

extern crate clap;
use clap::{App, Arg};

mod cartridge;
mod cpu;
mod emu;
mod gb;
mod memory;
mod mmu;
mod reg;
mod util;

use emu::Emulator;

fn main() {
    let matches = App::new("gameboy.rust")
        .version("1.0")
        .author("Yuma Matsune <yuma.matsune@gmail.com>")
        .arg(
            Arg::with_name("file_path")
                .help("path to ROM file")
                .required(true),
        )
        .get_matches();
    if let Some(path) = matches.value_of("file_path") {
        let mut file = File::open(path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        Emulator::new(data).run();
    }
}
