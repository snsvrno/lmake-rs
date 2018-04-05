use std::path::PathBuf;

use ansi_term::Colour::{Red,Yellow,Blue};

use processing::compile::{get_library_path,get_library_latest_version};

use lpsettings;
use love::project::project;
use version::version::Version;

pub fn from_toml(des : &PathBuf) {
  //! goes through the toml file and compiles all the project libraries

  match lpsettings::get_raw_local(Some("project.libraries")) {
    None => { output_error!("No libraries defined in the local lovepack.toml."); }
    Some(subsetting) => {
      if subsetting.is_string() { output_error!("No libraries defined in the local lovepack.toml."); }
      else {

        // gets the project version
        let project_version : Option<Version> = if let Ok(version) = project::get_required_version(&PathBuf::from(".")) { Some(version) } else { None };

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
                match Version::from_str(&version) {
                  None => { output_error!("Failed to parse version, is this valid? {}",Red.paint(version.to_string())); },
                  Some(version) => { 
                    match get_library_latest_version(&name,&version) {
                      None => { output_error!("Cannot find library {} version {}.",Red.paint(name.to_string()),Yellow.paint(version.to_string())); }
                      Some(latest) => { 
                        match get_library_path(&name,&latest) {
                          None => { output_error!("Cannot compile {} ({}), library not found.",Red.paint(name.to_string()),Yellow.paint(version.to_string())); }
                          Some(library_path) => {
                            output_println!("Compiling library: {} ({})",Blue.paint(name.to_string()),Yellow.paint(latest.to_string()));
                            super::super::compile(&library_path,&des,false,&project_version);
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
  }

}