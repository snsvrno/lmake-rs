#[derive(Deserialize,Debug)]
#[serde(untagged)]
pub enum Multivalue {
  Array(Vec<Multivalue>),
  Text(String),
  Switch(bool),
}