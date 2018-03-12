extern crate toml;
extern crate clap;
extern crate rand; use rand::Rng;
extern crate ansi_term; use ansi_term::Colour::{Red,Yellow,Blue,Green};

#[macro_use]
extern crate output;
#[macro_use]
extern crate serde_derive;
extern crate lpsettings;
extern crate version;

pub mod interface;
mod compile;
mod lualib;
mod lualibdef;

use std::collections::HashMap;
use std::path::PathBuf;
use std::env;
use std::fs;
use std::io::Write;

pub static LIBDEFFILE : &str = "lib.toml";

pub fn compile(path : &PathBuf, dest : &PathBuf) -> Result<PathBuf,&'static str> {
  
  match compile::validate_lualib_path(&path) {
    false => { 
      output_error!("Path {} is not a valid lua lib: {}", Red.paint(path.display().to_string()),Yellow.paint(format!("{} not found",LIBDEFFILE)));
      return Err("Not a valid library");
    }
    true => { output_debug!("Valid lua library"); }
  }

  match lualib::get_lualib_settings(&path) {
    None => { output_error!("Error loading lualib definition file for {}",&path.display().to_string()); }
    Some(definition) => {

      // builds the output path, can either do
      // (1) the library name
      // (2) the library name and version with the --name-with-version switch
      // (3) an cli provided name with the --compiled-name switch with a value being the new name. can be with or without the .lua extension
      let mut compiled_file_path = dest.clone();
      match env::var("LMAKE_COMPILE_WITH_VERSION_IN_NAME") {
        Err(_) => { compiled_file_path.push(format!("{}.{}",&definition.name,"lua")); },
        Ok(_) => { compiled_file_path.push(format!("{}-{}.{}",&definition.name,&definition.version.to_string(),"lua")); }
      }
      
      // looks at the requires and processess them.
      let mut preload_hash : HashMap<String,String> = HashMap::new();
      let mut array_of_preloads : Vec<String> = Vec::new();

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
        

            let preload_text :String = create_random_preload_name(&definition.name);
            preload_hash.insert(file.clone(),preload_text.clone());

            output_debug!("Loading {} into {}",&src_path.display().to_string(),&preload_text);
            array_of_preloads.push(lualib::create_preload_string(&src_path,&preload_text));

          }
        }
      }

      // creates the compiled output file.
      output_debug!("Creating compiled library {} at {}",definition.to_string(),compiled_file_path.display().to_string());
      match fs::File::create(&compiled_file_path) {
        Err(error) => { output_error!("Could not create \'{}\': {}",Red.paint(compiled_file_path.display().to_string()),Yellow.paint(error.to_string())); },
        Ok(mut file) => { 

          // writes the preloaded stuff
          for prl in array_of_preloads {
            match file.write_all(&prl.as_bytes()) {
              Err(error) => { output_error!("Could not write to \'{}\': {}",Red.paint(compiled_file_path.display().to_string()),Yellow.paint(error.to_string())); },
              Ok(_) => { output_debug!("Wrote preload to \'{}\'",Green.paint(compiled_file_path.display().to_string()))}
            }
          }

          // writes the basefill stuff
          match file.write_all(definition.to_compiled_base_file(&preload_hash).as_bytes()) {
            Err(error) => { output_error!("Could not write to \'{}\': {}",Red.paint(compiled_file_path.display().to_string()),Yellow.paint(error.to_string())); },
            Ok(_) => { output_debug!("Wrote compiled lib base to \'{}\'",Green.paint(compiled_file_path.display().to_string()))}
          }
        }
      }
      

    }
  }

  Err("unimplemented")
}

fn create_random_preload_name(library_name:&str) -> String {
  let mut additative : String = "".to_string();

  for i in 0..24 {
    let c = rand::thread_rng().gen_range(0,9);
    additative = format!("{}{}",additative,c);
  }

  format!("{}-{}",&library_name,&additative)
}