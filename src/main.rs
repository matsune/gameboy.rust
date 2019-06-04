use clap::{App, Arg};
use gameboy::emu::Emulator;

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
            Arg::with_name("sav_path")
                .short("s")
                .long("sav")
                .help("path to sav file")
                .required(false),
        )
        .get_matches();
    let file_path = matches.value_of("file_path").unwrap();
    let sav_path = matches.value_of("sav_path");

    Emulator::new(file_path).sav_path(sav_path).run(true);
}
