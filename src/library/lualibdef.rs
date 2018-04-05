use std::collections::HashMap;
use version::version::Version;
use library::multivalue::Multivalue;

#[derive(Deserialize)]
pub struct LibraryDefinition {
  pub name : String,
  pub user : String,
  pub author : String,
  pub email : Option<String>,
  pub version : Version,
  pub love : Option<Version>,

  pub upstream : Option<String>,
  pub requires : Option<HashMap<String,String>>,
  pub dependencies : Option<HashMap<String,HashMap<String,String>>>,
  pub options : Option<HashMap<String,Multivalue>>

}

impl LibraryDefinition {
  pub fn to_string(&self) -> String {
    format!("{}/{} ({})",&self.user,&self.name,&self.version.to_string())
  }

  pub fn to_compiled_base_file(&self,preload_hash : &HashMap<String,String>) -> String {
    //! creates the text block for the library in lua. the `local library .......... return library` part.
    
    // base information
    let mut info = format!("library.name = '{}'\nlibrary.user = '{}'\nlibrary.author = '{}'\nlibrary.version = '{}'",
      &self.name,&self.user,&self.author,&self.version.to_string()
    );

    // for "_" if its used
    let mut library_inital : Option<String> = None;

    // for the other requires, so they can be sorted and processed
    let mut requires_list : Vec<(String,String)> = Vec::new();
    let mut empty_requires : Vec<String> = Vec::new();

    // requires
    // adds each require to the list
    match self.requires {
      None => { },
      Some(ref hash) => { 
        for (entry,file) in hash.iter() {
          let new_require_name : String = if let Some(name) = preload_hash.get(file) { name.clone() } else { file.clone() };

          if entry == "_" {
            library_inital = Some(format!("local library = require (\"{}\")",&new_require_name));
          } else {
            // splits the name to que up empty tables to make
            let mut split : Vec<&str> = entry.split(".").collect();
            while split.len() > 0 {
              let temp_word = split.join(".");
              if !empty_requires.contains(&temp_word) {
                empty_requires.insert(0,temp_word);
              }
              split.pop();
            }
            requires_list.push((entry.to_string(),new_require_name));
          }
        }
      }
    }

    // inserts the empty stuff
    for part in empty_requires {
      // checks if we should create the line or not
      let mut create_me = true;
      for required in &requires_list { if part == required.0 { create_me = false; }}
    
      if create_me  {
        info = format!("{}\nlibrary.{} = {{}}",info,part);
      }
    }

    // inserts the remaining things into the info
    for part in requires_list {
      info = format!("{}\nlibrary.{} = require (\"{}\")",info,part.0,part.1);
    }

    if let Some(base) = library_inital {
      format!("{}\n{}\nreturn library\n",base,info)
    } else {
      format!("local library = {{}}\n{}\nreturn library\n", info)
    }
  }

}