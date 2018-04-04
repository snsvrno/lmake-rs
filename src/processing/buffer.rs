use regex;

use std::env;
use std::collections::HashMap;

pub fn inject_comment_header(buffer : &mut String) {
  *buffer = format!("{}\n\
    -- built with {} ({}) <{}>\n\
    -- a tool for compiling lua libraries from multiple source files and dependencies\n\n",

    buffer,
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION"),
    "https://github.com/snsvrno/lmake-rs"
  );
}

pub fn inject_preloads(buffer : &mut String, array_of_preloads : &Vec<String>) {
  for prl in array_of_preloads {
    *buffer = format!("{}\n{}",

      buffer,
      &prl
    );
  }
}

pub fn inject_basefill(buffer : &mut String, preload_hash : &str) {
  *buffer = format!("{}\n{}",

    buffer,
    &preload_hash
  );
}

pub fn remove_comments(buffer : &mut String) {
  if let Ok(_) = env::var("LMAKE_REMOVE_COMMENTS") {
    let re = regex::Regex::new(r"--.*").unwrap();
    *buffer = re.replace_all(buffer,regex::NoExpand("")).into_owned();
  }
}

pub fn remove_blank_lines(buffer : &mut String) {
  let mut temp_buffer : String = String::new();
  let re = regex::Regex::new(r"^ *$").unwrap();

  for line in buffer.lines() {
    if !re.is_match(&line) {
      temp_buffer = format!("{}\n{}",temp_buffer,line);
    }
  }
  *buffer = temp_buffer;
}


pub fn process_depends_references(buffer : &mut String,preload_hash : &HashMap<String,String>) {

  let mut used_vars : Vec<String> = Vec::new();

  let re = regex::Regex::new(r"@([^.]*)").unwrap();
  
  // figures out the var names for all of these.
  for mtch in re.find_iter(&buffer) {
    used_vars.push(mtch.as_str()[1..].to_string());
  }

  for var in used_vars {
    if let Some(value) = preload_hash.get(&var) {
      *buffer = buffer.replace(
        &format!("@{}",&var),
        &format!(" require(\"{}\")",&value)
      );
    }
  }

}

pub fn process_internal_references(buffer : &mut String, requires : &Option<HashMap<String,String>>, preload_hash : &HashMap<String,String>) {
  //! replaces @references that are refering to an internal file. These are files in the source tree under 'requires' so if a toml looks like
  //!
  //! ```toml
  //! [requires]
  //! "defaults.place.two" = "src.functions"
  //! ```
  //! and `src.functions` has some functions, this part will replace `@defaults.place.two` with the preload for `src.functions`. If you reference
  //! `@defaults.place.two:megaFunction()` or `@defaults.place.two.otherVar` it will replace to `[PRELOAD]:megaFunction()` and `[PRELOAD].otherVar`

  let mut used_vars : Vec<String> = Vec::new();

  let re = regex::Regex::new(r"@([A-Za-z0-9._]*)").unwrap();
  
  // figures out the var names for all of these.
  for mtch in re.find_iter(&buffer) {
    used_vars.push(mtch.as_str()[1..].to_string());
  }

  if let Some(ref requires) = *requires {

    for var in used_vars {
      let mut keep_on_going : bool = true;
      let mut seperated_path : Vec<&str> = var.split(".").collect();

      // goes through the path backwards and checks if they are valid. can't tell the difference because @ref.ref.func and @ref.ref.ref so goes though all of them. 
      while keep_on_going {
        if let Some(file_path) = requires.get(&seperated_path.join(".")) {
          if let Some(preload_path) = preload_hash.get(file_path) {
            output_debug!("replacing internal reference {} to {}",seperated_path.join("."),preload_path);
            *buffer = buffer.replace(
              &format!("@{}",&seperated_path.join(".")),
              &format!("require(\"{}\")",&preload_path)
            );
            keep_on_going = false;
          } 
          
        } else { seperated_path.pop(); }
        if seperated_path.len() <= 0 { keep_on_going = false; }
      }
    }
    
  }


}