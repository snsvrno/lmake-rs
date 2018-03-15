use ansi_term::Colour::{Red,Yellow};

use std::collections::HashMap;
use std::path::PathBuf;

use library;
use LIBDEFFILE;

pub fn get_local_libraries(root_path : &PathBuf) -> HashMap<String,PathBuf> {
  let mut map : HashMap<String,PathBuf> = HashMap::new();

  if !root_path.exists() {
    output_error!("{} does not exists.",Red.paint(root_path.display().to_string()));
    return map;
  }

  match root_path.read_dir() {
    Err(error) => { output_error!("Error reading {}: {}",Red.paint(root_path.display().to_string()),Yellow.paint(error.to_string())); }
    Ok(iter) => {
      for folder in iter {
        match folder {
          Err(error) => { output_error!("Error reading folder: {}",Yellow.paint(error.to_string()))}
          Ok(entry) => {
            let mut lib_file_path = entry.path().clone();
            lib_file_path.push(LIBDEFFILE);

            if lib_file_path.exists() {
              if let Some(def) = library::luafile::get_lualib_settings(&entry.path()) {
                map.insert(def.name,entry.path().clone());
              }
            }

          }
        }
      }
    }
  }

  map
}