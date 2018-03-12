use clap;
use ansi_term::Colour::Blue;
use std::path::PathBuf;
use std::env;

use lpsettings;

pub fn process(matches : &clap::ArgMatches) -> Result<(),&'static str> {
  //! process function to be used with [CLAP.RS](https://clap.rs/)'s `.get_matches()`.
  //!
  //! should be called with the subset of matches from clap's `.get_matches()` if used as a subcommand, or all the matches if used as the main app.
  //!
  //! ```rust
  //! // example of using as a subcommand, this is called after .get_matches() 
  //! match app.subcommand() {
  //!   ("settings", Some(matches)) => { interface::process(matches); },
  //!   _ => {},
  //! }
  //! ```
  
  // success!

  match matches.subcommand() {
    ("compile", Some(sub_m)) => { return process_compile(&sub_m); }
    _ => { output_error!("Not matches found"); return Err("error"); }
  }
  
}

fn process_compile(matches : &clap::ArgMatches) -> Result<(),&'static str> {


  if matches.is_present("name-with-version") { env::set_var("LMAKE_COMPILE_WITH_VERSION_IN_NAME","true"); }

  let library_path : PathBuf = if let Some(lib) = matches.value_of("PATH") { PathBuf::from(lib) } else { PathBuf::from(".") };
  output_debug!("using {} as the library path",Blue.paint(library_path.display().to_string()));
  match library_path.exists() {
    true => { output_debug!("Path exists.");
      let mut destination_path = library_path.clone();
      destination_path.push(lpsettings::get_value_or("compile-folder","bin"));

      super::compile(&library_path, &destination_path);
    }
    false => { 
      output_debug!("Path does not exist!"); 
      return Err("Library path doesn't exist.");
    }
  }

  


  Ok (())
}

pub fn app() -> clap::App<'static,'static> {
  //! [CLAP.RS](https://clap.rs/) app for easy integration.
  //!
  //! Can be easily added to any CLAP app to extend funcionality.
  //!
  //! Using ***lpsettings*** by itself.
  //!
  //! ```rust
  //! let app = interface::app()
  //!   .get_matches();
  //!
  //! match interface::process(&app) {
  //!   Err(error) => { println!("{}",error); }
  //!   Ok(_) => { }
  //! }
  //! ```
  //!
  //! Using ***lpsettings*** as part of another app.
  //!
  //! ```rust
  //! let app = clap::App("newapp")
  //!   .subcommand(interface::app().name("settings"))
  //!   .get_matches();
  //!
  //! match app.subcommand() {
  //!   ("settings", Some(matches)) => { interface::process(matches); },
  //!   _ => {},
  //! }
  //! ```

  clap::App::new("lmake")

  // general application information
    .version(env!("CARGO_PKG_VERSION"))
    .author("snsvrno<snsvrno@tuta.io>")
    .about("Lovepack tool for compiling multiple .lua file libraries into a single .lua file.")
    .name("lmake")

  // switches

  // parameters

  // subapps
    .subcommand(clap::SubCommand::with_name("compile")
      .about("Compiles the library.")

      // arguements
      .arg(clap::Arg::with_name("PATH")
        .help("Path to library to compile")
        .value_name("PATH")
        .required(true)
        .index(1))

      // switches
      .arg(clap::Arg::with_name("name-with-version")
        .help("Include the version in the compiled name")
        .long("name-with-version"))
      .arg(clap::Arg::with_name("compiled-name")
        .help("Set what to name the compiled file")
        .long("compiled-name")
        .short("c"))
      
    )

}