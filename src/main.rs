use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

extern crate clap;
use clap::{App, Arg};

mod cart;

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
        run(path).unwrap();
    }
}

fn run<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    cart::load_cartridge(data);
    Ok(())
}
