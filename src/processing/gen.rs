use rand;
use rand::Rng;

pub fn create_random_preload_name(library_name:&str) -> String {
  let mut additative : String = "".to_string();

  for _ in 0..24 {
    let c = rand::thread_rng().gen_range(0,9);
    additative = format!("{}{}",additative,c);
  }

  format!("{}-{}",&library_name,&additative)
}