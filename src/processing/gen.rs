use rand;
use rand::Rng;

use std::env;

use library::lualibdef::LibraryDefinition;

pub fn create_random_preload_name(library_name:&str) -> String {
  let mut additative : String = "".to_string();

  for _ in 0..24 {
    let c = rand::thread_rng().gen_range(0,9);
    additative = format!("{}{}",additative,c);
  }

  format!("{}-{}",&library_name,&additative)
}

pub fn compiled_file_name(def : &LibraryDefinition, dep : bool) -> String {
  if dep { return format!("{}-{}.{}",&def.name,&def.version.to_string(),"lua"); }
  else {
    if let Ok(new_name) = env::var("LMAKE_COMPILE_NAME") {
      return format!("{}.{}",new_name,"lua");
    } else if let Ok(_) = env::var("LMAKE_COMPILE_WITH_VERSION_IN_NAME") { 
      return format!("{}-{}.{}",&def.name,&def.version.to_string(),"lua");
    } else { 
      return format!("{}.{}",&def.name,"lua");
    }
  }
}