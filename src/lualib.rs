use std::path::PathBuf;
use std::fs;
use std::io::prelude::*;

use lualibdef::LibraryDefinition;

use ansi_term::Colour::{Yellow,Red,Green,Blue};
use toml;

pub fn get_lualib_settings(library_root_path : &PathBuf) -> Option<LibraryDefinition> {
  //! loads the lib.toml file into a LibraryDefinition and returns that

  let mut path_lib_def_file = library_root_path.clone();
  path_lib_def_file.push(super::LIBDEFFILE);

  let raw_lib_def_contents = get_raw_file_contents(&path_lib_def_file);
  let definition : Result<LibraryDefinition,toml::de::Error> = toml::from_str(&raw_lib_def_contents);

  match definition {
    Err(error) => { output_debug!("Error parsing the library definition file \'{}\': {}",Yellow.paint(super::LIBDEFFILE),Red.paint(error.to_string())); return None; }
    Ok(def) => {
      output_debug!("Loaded the library {}",def.to_string());
      return Some(def);
    }
  }
}

pub fn create_preload_string(path : &PathBuf,prename : &str) -> String {
  let contents = get_raw_file_contents(&path);

  format!("package.preload['{}'] = (function(...)\n\
    {}\n\
    end)\n\
    ",prename,contents)
}

fn get_raw_file_contents(path : &PathBuf) -> String {

  let mut file_contents = String::new();
  let file = fs::File::open(&path);
  match file { 
    Err(error) => { output_error!("Could not open \'{}\': {}",Red.paint(path.display().to_string()),Yellow.paint(error.to_string())); }
    Ok(mut file) => {
      match file.read_to_string(&mut file_contents){
        Err(error) => { output_error!("Saving of file \'{}\' contents to buffer failed: {}",Red.paint(path.display().to_string()),Yellow.paint(error.to_string())); }
        Ok(_) => { output_debug!("Saving of contents to buffer succeeded."); }
      }
    }
  }

  file_contents
}