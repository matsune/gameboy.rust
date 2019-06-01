use clap::{App, Arg};
use gameboy::emu;
use std::fs::File;
use std::io::Read;

fn main() {
    let matches = App::new("gameboy.rust")
        .version("1.0")
        .author("Yuma Matsune <yuma.matsune@gmail.com>")
        .arg(
            Arg::with_name("file_path")
                .help("path to ROM file")
                .required(true),
        )
        .arg(
            Arg::with_name("bootrom")
                .short("b")
                .long("bootrom")
                .help("run bootrom"),
        )
        .get_matches();
    let path = matches.value_of("file_path").unwrap();
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    let bootrom = matches.is_present("bootrom");
    emu::run(data, !bootrom);
}
