use std::path::{PathBuf,Path};
use ansi_term::Colour::{Red,Yellow};

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
      lib_file.push(super::LIBDEFFILE);
      match lib_file.exists() {
        false => {
          output_error!("Folder isn't formatted correctly, no {} file found in \'{}\'",Yellow.paint(super::LIBDEFFILE),Red.paint(library_root_path.display().to_string()));
          false
        },
        true => { true }
      }
    },
  }
}

