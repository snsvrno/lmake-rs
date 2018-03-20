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
    ("install", Some(sub_m)) => { return process_install(&sub_m); }
    _ => { output_error!("Not matches found"); return Err("error"); }
  }
  
}

fn process_install(matches : &clap::ArgMatches) -> Result<(),&'static str> {

  match matches.subcommand() {
    ("add", Some(sub_m)) => { process_install_add(&sub_m); }
    _ => { process_install_none(&matches); }
  }

  Ok ( () )
}

fn process_install_none(matches : &clap::ArgMatches) -> Result<(),&'static str> {
  if let Some(path) = matches.value_of("PATH") {
    println!("{}",path);
  }
  Ok ( () )
}

fn process_install_add(matches : &clap::ArgMatches) -> Result<(),&'static str> {
  if let Some(libraries) = matches.values_of("LIBRARY") {
    for lib in libraries {
      
      let mut vec : Vec<&str> = lib.split("$").collect();
      let var : Option<String> = if vec.len() > 1 { Some(vec[1].to_string()) } else { None };

      let mut vec_2 : Vec<&str>  = vec[0].split(":").collect();
      let version : Option<String> = if vec_2.len() > 1 { Some(vec_2[1].to_string()) } else { Some("latest".to_string()) };
      let name : String = vec_2[0].to_string();

      if var.is_some() {
        lpsettings::set_value_local(&format!("project.libraries.{}.version",&name),&version.unwrap());
        lpsettings::set_value_local(&format!("project.libraries.{}.var",&name),&var.unwrap());
      } else {
        lpsettings::set_value_local(&format!("project.libraries.{}",&name),&version.unwrap());
      }
    }
  }
  Ok ( () )
}

fn process_compile(matches : &clap::ArgMatches) -> Result<(),&'static str> {


  if matches.is_present("name-with-version") || lpsettings::get_value_or("lmake.name-with-version","false") == "true" { env::set_var("LMAKE_COMPILE_WITH_VERSION_IN_NAME","true"); }
  if matches.is_present("remove-comments") || lpsettings::get_value_or("lmake.remove-comments","false") == "true" { env::set_var("LMAKE_REMOVE_COMMENTS","true"); } 

  if let Some(new_name) = matches.value_of("compiled-name") { env::set_var("LMAKE_COMPILE_NAME",new_name); }

  let library_path : PathBuf = if let Some(lib) = matches.value_of("PATH") { PathBuf::from(lib) } else { PathBuf::from(".") };
  output_debug!("using {} as the library path",Blue.paint(library_path.display().to_string()));
  match library_path.exists() {
    true => { output_debug!("Path exists.");
      let mut destination_path = library_path.clone();
      destination_path.push(lpsettings::get_value_or("compile-folder","bin"));

      match super::compile(&library_path, &destination_path, false) {
        Err(error) => { output_error!("Error compiling: {}",error.to_string()); }
        Ok(path) => { println!("Successfully compiled: {}",Blue.paint(path.display().to_string())); }
      }
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

  // INSTALL subapp
    .subcommand(clap::SubCommand::with_name("install")
      .about("Compiles and Installs libraries based on a definition file.")

    // add subcommand
      .subcommand(clap::SubCommand::with_name("add")
        .about("Adds librares to a definition file.")
        .arg(clap::Arg::with_name("path")
          .help("Path to definition file")
          .short("p")
          .long("path"))
        .arg(clap::Arg::with_name("LIBRARY")
          .help("Library === name:version$var.")
          .value_name("LIBRARY")
          .required(true)
          .multiple(true))
        )

    // arguements
      .arg(clap::Arg::with_name("PATH")
        .help("Path to definition file")
        .value_name("PATH")
        .index(1))
      )


  // COMPILE subapp
    .subcommand(clap::SubCommand::with_name("compile")
      .about("Compiles the library.")

    // arguements
      .arg(clap::Arg::with_name("PATH")
        .help("Path to library to compile")
        .value_name("PATH")
        .required(true))

    // switches
      .arg(clap::Arg::with_name("name-with-version")
        .help("Include the version in the compiled name")
        .long("name-with-version"))

      .arg(clap::Arg::with_name("remove-comments")
        .help("Removes all comments from files")
        .long("remove-comments"))

    // parameters
      .arg(clap::Arg::with_name("compiled-name")
        .help("Set what to name the compiled file")
        .long("compiled-name")
        .short("c")
        .takes_value(true))
      
    )

}