extern crate toml;
extern crate clap;
extern crate regex;
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

      println!("Compiling {}/{} ({})",Blue.paint(definition.user.clone()),Blue.paint(definition.name.clone()),Yellow.paint(definition.version.to_string().clone()));

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

          let mut successful_build = true;

          if !inject_comment_header(&mut file) { successful_build = false; }               // writes a comment about the library
          if !inject_preloads(&mut file,&array_of_preloads) { successful_build = false; }  // writes the preloaded stuff          
          if !inject_basefill(&mut file,&definition.to_compiled_base_file(&preload_hash)) { successful_build = false; }  // writes the basefill stuff

          if successful_build { println!("Compilation {}!",Green.paint("Successful")); } 
          else { println!("Compilation {}!",Red.paint("Failed")); }
        } 
      }
      

    }
  }

  Err("unimplemented")
}

fn inject_comment_header(file : &mut fs::File) -> bool {
  output_debug!("Writing header comment block ..");

  match file.write_all(format!("\
    -- built with {} ({}) <{}>\n\
    -- a tool for compiling lua libraries from multiple source files and dependencies\n\n",
      env!("CARGO_PKG_NAME"),
      env!("CARGO_PKG_VERSION"),
      "https://github.com/snsvrno/lmake-rs"
    ).as_bytes()) {
      
      Err(error) => { 
        output_error!(" .. could not write to file: {}",Yellow.paint(error.to_string())); 
        return false;
      },
      Ok(_) => { 
        output_debug!(" .. wrote comment header.");
        return true;
      }
  }
}
fn inject_preloads(file : &mut fs::File,array_of_preloads : &Vec<String>) -> bool {
  for prl in array_of_preloads {
    output_debug!("Writing preloads ..");

    let mut preload_blob : String = prl.clone();

    // removes the comments
    if let Ok(_) = env::var("LMAKE_REMOVE_COMMENTS") {
      output_debug!(" .. removing comments.");
      let re = regex::Regex::new(r"--.*").unwrap();
      preload_blob = re.replace_all(&preload_blob,regex::NoExpand("")).into_owned();
    }

    // removes empty lines that have spaces
    {
      output_debug!(" .. removing empty space lines.");
      let re = regex::Regex::new(r" *\n").unwrap();
      preload_blob = re.replace_all(&preload_blob,regex::NoExpand("")).into_owned();
    }

    match file.write_all(&preload_blob.as_bytes()) {
      Err(error) => { 
        output_error!(" .. could not write to file: {}",Yellow.paint(error.to_string())); 
        return false; 
      },
      Ok(_) => { 
        output_debug!(" .. wrote preload.");
      }
    }
  }
  return true;
}
fn inject_basefill(file : &mut fs::File,preload_hash : &str) -> bool {
  output_debug!("Writing base fill section ..");

  match file.write_all(preload_hash.as_bytes()) {
    Err(error) => { 
      output_error!(" .. could not write to file: {}",Yellow.paint(error.to_string())); 
      return false; 
    },
    Ok(_) => { 
      output_debug!(" .. wrote compiled lib base.");
      return true;
    }
  }
}


fn create_random_preload_name(library_name:&str) -> String {
  let mut additative : String = "".to_string();

  for _ in 0..24 {
    let c = rand::thread_rng().gen_range(0,9);
    additative = format!("{}{}",additative,c);
  }

  format!("{}-{}",&library_name,&additative)
}