extern crate clap;
extern crate sereal_decoder;

use std::io::stdout;
use std::fs::File;
use std::io::Write;

use clap::App;
use clap::Arg;

use sereal_decoder::arc::ArcBuilder;
use sereal_decoder::parse;
use sereal_decoder::Error;

fn main() {
    let matches = App::new("sereal-dump")
        .version("0.1.0")
        .arg(Arg::with_name("input")
             .required(true)
             .index(1))
        .get_matches();

    let fname = matches.value_of("input").unwrap();
    if let Err(err) = process(fname) {
        writeln!(stdout(), "{}: {:?}", fname, err).unwrap();
    }
}

fn process(name: &str) -> Result<(), Error> {
    let file = File::open(name)?;
    let value = parse(file, ArcBuilder)?;
    println!("{:#?}", value);
    Ok(())
}
