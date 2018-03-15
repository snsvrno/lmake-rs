extern crate toml;
extern crate clap;
extern crate regex;
extern crate rand; 
extern crate ansi_term; use ansi_term::Colour::{Red,Yellow,Blue,Green};
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate output;
extern crate lpsettings;
extern crate version;

use std::collections::HashMap;
use std::path::PathBuf;
use std::env;
use std::fs;
use std::io::Write;

pub mod interface;
mod processing;
mod library;
mod local;

pub static LIBDEFFILE : &str = "lib.toml";

pub fn compile(path : &PathBuf, dest : &PathBuf) -> Result<PathBuf,&'static str> {

  let mut final_compiled_path : Option<PathBuf> = None;

  match processing::compile::validate_lualib_path(&path) {
    false => { 
      output_error!("Path {} is not a valid lua lib: {}", Red.paint(path.display().to_string()),Yellow.paint(format!("{} not found",LIBDEFFILE)));
      return Err("Not a valid library");
    }
    true => { output_debug!("Valid lua library"); }
  }

  match library::luafile::get_lualib_settings(&path) {
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
        

            let preload_text :String = processing::gen::create_random_preload_name(&definition.name);
            preload_hash.insert(file.clone(),preload_text.clone());

            output_debug!("Loading {} into {}",&src_path.display().to_string(),&preload_text);
            array_of_preloads.push(library::luafile::create_preload_string(&src_path,&preload_text));

          }
        }
      }

      match definition.dependencies {
        None => { },
        Some(ref hash) => {
          for(name,blob) in hash.iter() {

            let library_name : String = if let Some(lname) = blob.get("name") { lname.clone() } else { name.clone() };
            let reference_name : String = name.clone();

            if let Some(dependancy_path) = get_library_path(&library_name) {
              output_debug!("Found library at {}",Blue.paint(dependancy_path.display().to_string()));

              match compile(&dependancy_path,&dest) {
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
            } else {
              output_error!("Cannot find library {} ",Red.paint(library_name.clone()));
            }
          }
        }
      }

      // creates the compiled output file.
      output_debug!("Creating compiled library {} at {}",definition.to_string(),compiled_file_path.display().to_string());
      match fs::File::create(&compiled_file_path) {
        Err(error) => { output_error!("Could not create \'{}\': {}",Red.paint(compiled_file_path.display().to_string()),Yellow.paint(error.to_string())); },
        Ok(mut file) => { 

          let mut successful_build = true;
          let mut file_buffer : String = String::new();

          // adding the dependencies and requires source code
          processing::buffer::inject_comment_header(&mut file_buffer);
          processing::buffer::inject_preloads(&mut file_buffer,&array_of_preloads);  // writes the preloaded stuff          
          processing::buffer::inject_basefill(&mut file_buffer,&definition.to_compiled_base_file(&preload_hash));  // writes the basefill stuff

          // processess the resulting buffer
          processing::buffer::remove_comments(&mut file_buffer);
          processing::buffer::remove_blank_lines(&mut file_buffer);
          processing::buffer::process_depends_references(&mut file_buffer,&preload_hash); // takes all the @ references and replaces them.

          // writes teh buffer to the file
          match file.write_all(&file_buffer.as_bytes()) {
            Err(error) => { output_error!(" .. could not write to file: {}",Yellow.paint(error.to_string())); },
            Ok(_) => { output_debug!("Wrote library to file."); }
          }

          let result_text = if successful_build { 
            final_compiled_path = Some(compiled_file_path.clone());
            Green.paint("Successful")
          } else { 
            Red.paint("Failed") 
          };
          output_debug!("Compiling {}/{} ({}): {}",Blue.paint(definition.user.clone()),Blue.paint(definition.name.clone()),Yellow.paint(definition.version.to_string().clone()),result_text);
        } 
      }
      

    }
  }

  if let Some(path) = final_compiled_path { return Ok(path); }
  Err("General compilation error.")
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