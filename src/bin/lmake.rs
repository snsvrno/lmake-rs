//! simple basic cli interface for lamke library
extern crate clap;
extern crate lmake;
extern crate ansi_term;

use std::env;
use lmake::interface;

fn main() {  
  // builds the app
  let app = interface::app()
    .arg(clap::Arg::with_name("debug").long("debug").help("Shows additional information about commands run."))
    .get_matches();

  // this will be in the parent program, so its only here in the bin app
  if app.is_present("debug") { env::set_var("OUTPUT_DEBUG_ENABLED","true"); }

  // processess the arguement matches.
  match interface::process(&app) {
    Err(error) => { println!("{}",error); }
    Ok(_) => { }
  }
}