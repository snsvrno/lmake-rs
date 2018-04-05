use ansi_term::Colour::{Red,Yellow,Blue};

use std::path::PathBuf;
use std::collections::HashMap;

use LIBDEFFILE;
use processing;
use local;
use library;
use library::lualibdef::LibraryDefinition;
use version::version::Version;
use lpsettings;
use git2;

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

      // gets the required version / with error checking on MAX!
      let required_version : Version = if let Some(ver) = blob.get("version") { 
        if let Some(version) = Version::from_str(ver) { version } 
        else { 
          output_error!("Malformed version requirement in dependency for {}: {}, using \"{}\" instead.",Blue.paint(library_name.to_string()),Red.paint(ver.to_string()),Yellow.paint("*"));
          Version::from_str("*").unwrap()
        }
      } else { Version::from_str("*").unwrap() };

      let reference_name : String = name.clone();

      if let Some(dependancy_path) = get_library_path(&library_name,&required_version) {
        output_debug!("Found library at {}",Blue.paint(dependancy_path.display().to_string()));

        match super::super::compile(&dependancy_path,&dest,true,&None) {
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

pub fn get_library_latest_version(library_name:&str, version:&Version) -> Option<Version> {
  // checks locally.
  if let Some(value) = lpsettings::get_value("library.local-folder") { 
    let libraries : HashMap<String,PathBuf> = local::library::get_local_libraries(&PathBuf::from(&value));
    // it we find the library locally
    if let Some(path) = libraries.get(library_name) { 
      // now we need to check if it has the right version inside it.
      let version_tags = get_tag_names(&path);
      match version.latest_compatible(&version_tags) {
        None => { output_error!("No version found matching {} requirements.",Red.paint(version.to_string()));}
        Some(matching_version) => { 
          output_debug!("Found {} locally.",Yellow.paint(matching_version.clone()));
          return Version::from_str(matching_version);
        }
      }
    }
  }
  None
}

pub fn get_library_path(library_name:&str, version:&Version) -> Option<PathBuf> {
  //! looks for the correct path to the library requested.
  //!
  //! First it will look in the local area, check for git tags, and then mark the best matching tag.
  //! Then it checks remotely (not yet implemented)
  //!
  //! Finally it then clones, checkouts the tag, and then returns that path to be used.

  // checks if it is locally
  let mut best_local_version : Option<(String,PathBuf)> = None;

  // checks locally.
  if let Some(value) = lpsettings::get_value("library.local-folder") { 
    let libraries : HashMap<String,PathBuf> = local::library::get_local_libraries(&PathBuf::from(&value));
    // it we find the library locally
    if let Some(path) = libraries.get(library_name) { 
      // now we need to check if it has the right version inside it.
      let version_tags = get_tag_names(&path);
      match version.latest_compatible(&version_tags) {
        None => { output_error!("No version found matching {} requirements.",Red.paint(version.to_string()));}
        Some(matching_version) => { 
          output_debug!("Found {} locally.",Yellow.paint(matching_version.clone()));
          best_local_version = Some((matching_version.to_string(),path.clone())); 
        }
      }
    }
  }

  // checks remotely ....
  output_debug!("remote libraries not yet implemented.");

  // finds the path
  if let Some((matching_version,path)) = best_local_version {
    output_debug!("Using {} for the requirement {}",Yellow.paint(matching_version.clone()),Blue.paint(version.to_string()));
    let mut cloned_path = if let Ok(path) = lpsettings::get_settings_folder() { path } else { PathBuf::from(".") };

    cloned_path.push(lpsettings::get_value_or("core.cache","cache"));
    cloned_path.push(format!("{}-{}",&library_name,&matching_version.to_string()));
    if cloned_path.exists() { 
      output_debug!("{} already exists, using existing.",Blue.paint(cloned_path.display().to_string()));
      return Some(cloned_path); 
    }
    match clone_repository(&path,&cloned_path) {
      Err(error) => { output_error!("Cannot clone to {}: {}",Red.paint(cloned_path.display().to_string()),Yellow.paint(error.to_string())); }
      Ok(_) => {
        match checkout_tag(&cloned_path,&matching_version) {
          Err(error) => { output_error!("Cannot checkout tag {}: {}",Red.paint(matching_version.clone()),Yellow.paint(error.to_string())); }
          Ok(_) => { 
            return Some(cloned_path);
          }
        }
      }
    }
  }
  
  output_error!("No local library path set, please set value {} in order to use.",Red.paint("library.local-folder"));
  None
}

// GIT STUFF

fn get_tag_names(src : &PathBuf) -> Vec<String> {
  //! gets list of all the tags for the given repository path

  let mut tags : Vec<String> = Vec::new();

  match git2::Repository::open(&src.display().to_string()) {
    Err(error) => { output_error!("Error getting tags for repository {}: {}",Red.paint(src.display().to_string()),Yellow.paint(error.to_string())); }
    Ok(repo) => {  
      if let Ok(tags_from_git2) = repo.tag_names(None) {
        for option_tags in tags_from_git2.iter() {
          if let Some(tag) = option_tags {
            tags.push(tag.to_string());
          }
        }
      }
    }
  }
  
  tags
}

fn clone_repository(src : &PathBuf, des : &PathBuf) -> Result<(),git2::Error> {
  //! copies the repository form the SRC to the DES

  match git2::Repository::clone(&src.display().to_string(),&des.display().to_string()) {
    Err(error) => { Err(error) }
    Ok(_repo) => { Ok( () ) }
  }
}

fn checkout_tag(src :&PathBuf, tag : &str) -> Result<(),git2::Error> {
  //! check out the src repository at the given tag.

  match git2::Repository::open(&src.display().to_string()) {
    Err(error) => { Err(error) }
    Ok(repo) => {  
      
      match repo.find_reference(&format!("refs/tags/{}",&tag)) {
        Err(error) => { Err(error) }
        Ok(reference) => { 

          match repo.set_head(&reference.name().unwrap()) {
            Err(error) => { Err(error) }
            Ok(_) => { 

              match repo.checkout_head(None) {
                Err(error) => { Err(error) }
                Ok(_) => { 
                  Ok( () )
                }
              }

            }
          }

        }
      }

    }
  }
}