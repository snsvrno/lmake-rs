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

  let re = regex::Regex::new(r" @([^.]*)").unwrap();
  
  // figures out the var names for all of these.
  for mtch in re.find_iter(&buffer) {
    used_vars.push(mtch.as_str()[2..].to_string());
  }

  for var in used_vars {
    if let Some(value) = preload_hash.get(&var) {
      *buffer = buffer.replace(
        &format!(" @{}",&var),
        &format!(" require(\"{}\")",&value)
      );
    }
  }

}