use std::collections::HashMap;
use version::version::Version;

#[derive(Deserialize)]
pub struct LibraryDefinition {
  pub name : String,
  pub user : String,
  pub author : String,
  pub email : Option<String>,
  pub version : Version,

  pub upstream : Option<String>,
  pub requires : Option<HashMap<String,String>>,

}

impl LibraryDefinition {
  pub fn to_string(&self) -> String {
    format!("{}/{} ({})",&self.user,&self.name,&self.version.to_string())
  }

  pub fn to_compiled_base_file(&self,preload_hash : &HashMap<String,String>) -> String {
    // base information
    let mut info = format!("library.name = '{}'\nlibrary.user = '{}'\nlibrary.author = '{}'\nlibrary.version = '{}'",
      &self.name,&self.user,&self.author,&self.version.to_string()
    );

    // for "_" if its used
    let mut library_inital : Option<String> = None;

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
            info = format!("{}\nlibrary.{} = require (\"{}\")",
              info,entry,&new_require_name);
          }
        }
      }
    }
    if let Some(base) = library_inital {
      format!("{}\n{}\nreturn library",base,info)
    } else {
      format!("local library = {{}}\n{}\nreturn library", info)
    }
  }

}