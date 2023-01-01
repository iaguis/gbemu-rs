use gbemu_rs::{Emulator,Config};

use std::{env,process};

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        process::exit(1);
    });

    let mut e = Emulator::new(config);
    e.run();
}
