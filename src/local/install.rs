use std::path::PathBuf;

use ansi_term::Colour::{Red,Yellow,Blue};

use processing::compile::get_library_path;

use lpsettings;
use version::version::Version;

pub fn from_toml(des : &PathBuf) {
  //! goes through the toml file and compiles all the project libraries

  match lpsettings::get_raw_local(Some("project.libraries")) {
    None => { output_error!("No libraries defined in the local lovepack.toml."); }
    Some(subsetting) => {
      if subsetting.is_string() { output_error!("No libraries defined in the local lovepack.toml."); }
      else {
        if let Some(hash) = subsetting.to_hash() {
          for (name,val) in hash.iter() { 
            
            // gets the version, checks for a simple or complex subsetting
            let version : Option<String> = if val.is_string() { val.to_string() } else { 
              match val.to_hash().unwrap().get("version") {
                None => { None },
                Some(subsetting) => { subsetting.to_string() }
            }};

            match version {
              None => { output_error!("No version defined for {}",Red.paint(name.to_string())); }
              Some(version) => { 
                // compiles this
                output_println!("Compiling library: {} ({})",Blue.paint(name.to_string()),Yellow.paint(version.to_string()));
                match Version::from_str(&version) {
                  None => { output_error!("Failed to parse version, is this valid? {}",Red.paint(version.to_string())); },
                  Some(version) => { 
                    match get_library_path(&name,&version) {
                      None => { output_error!("Cannot compile {} ({}), library not found.",Red.paint(name.to_string()),Yellow.paint(version.to_string())); }
                      Some(library_path) => {
                        super::super::compile(&library_path,&des,false);
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

}