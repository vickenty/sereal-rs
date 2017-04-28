extern crate clap;
extern crate sereal_decoder;

use std::io::stdout;
use std::fs::File;
use std::io::Write;

use clap::App;
use clap::Arg;

use sereal_decoder::arena::{ Arena, ArenaBuilder };
use sereal_decoder::parse;
use sereal_decoder::Error;

fn main() {
    let matches = App::new("sereal-dump")
        .version("0.1.0")
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .help("do not dump the contents, just parse the file"))
        .arg(Arg::with_name("input")
            .required(true)
            .index(1))
        .get_matches();

    let fname = matches.value_of("input").unwrap();
    let quiet = matches.is_present("quiet");

    if let Err(err) = process(fname, quiet) {
        writeln!(stdout(), "{}: {:?}", fname, err).unwrap();
    }
}

fn process(name: &str, quiet: bool) -> Result<(), Error> {
    let file = File::open(name)?;
    let mut buf = Vec::new();
    let arena = Arena::new();
    let value = parse(file, ArenaBuilder::new(&arena), &mut buf)?;

    if !quiet {
        println!("{:#?}", value);
    }

    Ok(())
}
