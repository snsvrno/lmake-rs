use ansi_term::Colour::{Red,Yellow,Blue};

use std::path::PathBuf;
use std::collections::HashMap;

use LIBDEFFILE;
use processing;
use local;
use library;
use library::lualibdef::LibraryDefinition;
use lpsettings;

pub fn validate_lualib_path(library_root_path : &PathBuf) -> bool {
  //! checks to see if the supplied path has a library inside of it.
  //! the lib.toml file is what defines a library (or whatever is LIBDEFFILE)
  match library_root_path.exists() {
    false => {
      output_error!("Given library path \'{}\' does not exists.",Red.paint(library_root_path.display().to_string()));
      false
    },
    true => {
      let mut lib_file : PathBuf = library_root_path.clone();
      lib_file.push(LIBDEFFILE);
      match lib_file.exists() {
        false => {
          output_error!("Folder isn't formatted correctly, no {} file found in \'{}\'",Yellow.paint(LIBDEFFILE),Red.paint(library_root_path.display().to_string()));
          false
        },
        true => { true }
      }
    },
  }
}

pub fn requires(path : &PathBuf, definition : &LibraryDefinition,array_of_preloads : &mut Vec<String>,preload_hash : &mut HashMap<String,String>) {
  match definition.requires {
    None => { },
    Some(ref hash) => {
      // for each requirement in the definition
      for (_,file) in hash.iter() {
        // builds the path to the file
        let mut src_path = path.clone();
        let temp_vector : Vec<&str> = file.split(".").collect();
        for cc in 0..temp_vector.len() { 
          if cc == (temp_vector.len()-1) { 
            src_path.push(format!("{}.{}",temp_vector[cc],"lua"));
          } else {
            src_path.push(temp_vector[cc]);
          }
        }

        let preload_text :String = processing::gen::create_random_preload_name(&definition.name);
        preload_hash.insert(file.clone(),preload_text.clone());

        output_debug!("Loading {} into {}",&src_path.display().to_string(),&preload_text);
        array_of_preloads.push(library::luafile::create_preload_string(&src_path,&preload_text));
      }
    }
  }
}

pub fn dependencies(dest : &PathBuf, definition : &LibraryDefinition,array_of_preloads : &mut Vec<String>,preload_hash : &mut HashMap<String,String>) {
  if let Some(ref hash) = definition.dependencies {

    for(name,blob) in hash.iter() {
      let library_name : String = if let Some(lname) = blob.get("name") { lname.clone() } else { name.clone() };
      let reference_name : String = name.clone();

      if let Some(dependancy_path) = get_library_path(&library_name) {
        output_debug!("Found library at {}",Blue.paint(dependancy_path.display().to_string()));

        match super::super::compile(&dependancy_path,&dest,true) {
          Err(error) => {
            output_error!("Error compiling dependancy {}: {}",Blue.paint(library_name.clone()),Yellow.paint(error.to_string()));
          },
          Ok(compiled_path) => { 
            // nead to insert the source into a preload
            let preload_text :String = processing::gen::create_random_preload_name(&definition.name);
            preload_hash.insert(reference_name.clone(),preload_text.clone());

            array_of_preloads.push(library::luafile::create_preload_string(&compiled_path,&preload_text));
          }
        }
      } else { output_error!("Cannot find library {} ",Red.paint(library_name.clone())); }
    }

  }
}

fn get_library_path(library_name:&str) -> Option<PathBuf> {
  if let Some(value) = lpsettings::get_value("library.local-folder") { 
    // first it checks locally.
    let libraries : HashMap<String,PathBuf> = local::library::get_local_libraries(&PathBuf::from(&value));
    if let Some(path) = libraries.get(library_name) { return Some(path.clone()); } else { return None; }
  } else {
      output_error!("No local library path set, please set value {} in order to use.",Red.paint("library.local-folder"));
  }
  None
}