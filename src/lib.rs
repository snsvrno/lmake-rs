extern crate toml;
extern crate clap;
extern crate regex;
extern crate rand; 
extern crate git2;
extern crate ansi_term; use ansi_term::Colour::{Red,Yellow,Blue,Green};
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate output;
extern crate love;
extern crate lpsettings;
extern crate version; use version::version::Version;
extern crate base64;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::io::Write;

pub mod interface;
mod processing;
mod library;
mod local;

pub static LIBDEFFILE : &str = "lib.toml";

pub fn compile(path : &PathBuf, dest : &PathBuf, dep : bool, version : &Option<Version>) -> Result<PathBuf,&'static str> {

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

      // first will check if the library is compatible with the version of love being used.
      // Right now it only makes a warning
      if let Some(ref version) = *version {
        if let Some(ref req_ver) = definition.love {
          if !version.is_compatible_with(req_ver) {
            output_warning!("{} has a LOVE requirement of {} and is not listed as compatible with LOVE {}",definition.name,req_ver,version.to_string());
          }
        }
      }

      // builds the output path, can either do
      // (1) the library name
      // (2) the library name and version with the --name-with-version switch
      // (3) an cli provided name with the --compiled-name switch with a value being the new name. can be with or without the .lua extension
      // also puts the dependencies into a cache dep folder.
      let mut compiled_file_path = if dep { 
        let mut new_folder = if let Ok(mut folder) = lpsettings::get_settings_folder() { folder } else { PathBuf::from(".") };
        new_folder.push(lpsettings::get_value_or("core.cache","cache"));
        new_folder
      } else { dest.clone() };
      std::fs::create_dir_all(&compiled_file_path);
      compiled_file_path.push(processing::gen::compiled_file_name(&definition,dep));

      // looks at the requires and processess them.
      let mut preload_hash : HashMap<String,String> = HashMap::new();
      let mut array_of_preloads : Vec<String> = Vec::new();

      // processing the components of the definition file
      processing::compile::requires(&path,&definition,&mut array_of_preloads,&mut preload_hash);
      processing::compile::dependencies(&dest,&definition,&mut array_of_preloads,&mut preload_hash);

      // the buffer for the compiled library contents
      let mut file_buffer : String = String::new();

      // adding the dependencies and requires source code
      processing::buffer::inject_comment_header(&mut file_buffer);
      processing::buffer::inject_preloads(&mut file_buffer,&array_of_preloads);  // writes the preloaded stuff          
      processing::buffer::inject_basefill(&mut file_buffer,&definition.to_compiled_base_file(&preload_hash));  // writes the basefill stuff

      // processess the resulting buffer, formatting, var names, etc..
      processing::buffer::remove_comments(&mut file_buffer);
      processing::buffer::remove_blank_lines(&mut file_buffer);
      processing::buffer::process_depends_references(&mut file_buffer,&preload_hash); // takes all the @ references and replaces them if dependencies.
      processing::buffer::process_internal_references(&mut file_buffer,&definition.requires,&preload_hash); // takes all the @ references and replaces them if internal references.

      // does optional stuff, like asset replacement
      processing::buffer::embed_assets(&mut file_buffer,&definition.options);

      // creates the compiled output file.
      match fs::File::create(&compiled_file_path) {
        Err(error) => { output_error!("Could not create \'{}\': {}",Red.paint(compiled_file_path.display().to_string()),Yellow.paint(error.to_string())); },
        Ok(mut file) => { 
          // writes the buffer to the file
          let result_text = if let Err(error) = file.write_all(&file_buffer.as_bytes()) { format!("{}: {}",Red.paint("Failed"), Yellow.paint(error.to_string())) }
            else { 
              final_compiled_path = Some(compiled_file_path.clone());
              format!("{}",Green.paint("Successful"))
          };
          output_debug!("Compiling {}/{} ({}): {}",Blue.paint(definition.user.clone()),Blue.paint(definition.name.clone()),Yellow.paint(definition.version.to_string().clone()),result_text);
        } 
      }
      

    }
  }

  if let Some(path) = final_compiled_path { return Ok(path); }
  Err("General compilation error.")
}