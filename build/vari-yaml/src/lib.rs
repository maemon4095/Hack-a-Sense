use std::collections::HashMap;

pub enum Yaml {
    Value(String),
    Array(Vec<Yaml>),
    Hash(HashMap<String, Yaml>),
}

pub fn parse(source: String) -> Yaml {
    todo!()
}
