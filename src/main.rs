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
                .help("path to sav file"),
        )
        .arg(
            Arg::with_name("bootrom")
                .short("b")
                .long("bootrom")
                .help("run bootrom"),
        )
        .arg(
            Arg::with_name("mute")
                .short("m")
                .long("mute")
                .help("disable sound"),
        )
        .get_matches();
    let file_path = matches.value_of("file_path").unwrap();
    let sav_path = matches.value_of("sav_path");
    let mute = matches.is_present("mute");
    let bootrom = matches.is_present("bootrom");
    Emulator::new(file_path)
        .sav_path(sav_path)
        .mute(mute)
        .run(!bootrom);
}
