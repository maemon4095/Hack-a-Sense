use inline_collection::iterators::IteratorExtensions;
use std::collections::HashMap;
pub enum Yaml {
    Scalar(Option<String>),
    Sequence(Vec<Yaml>),
    Mapping(HashMap<Yaml, Yaml>),
}

pub enum Document {
    Single(Yaml),
    Multiple(Vec<Yaml>),
}

impl<'a> IntoIterator for &'a Document {
    type Item = &'a Yaml;

    type IntoIter = std::slice::Iter<'a, Yaml>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Document::Single(yaml) => std::slice::from_ref(yaml).iter(),
            Document::Multiple(vec) => vec.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a mut Document {
    type Item = &'a mut Yaml;

    type IntoIter = std::slice::IterMut<'a, Yaml>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Document::Single(yaml) => std::slice::from_mut(yaml).iter_mut(),
            Document::Multiple(vec) => vec.iter_mut(),
        }
    }
}

pub enum YamlParser {
    Initial,
    StringScalar { quote: char },
    LiteralScalar,
    FoldedScalar,
    AutoScalar,
    Sequence,
    Mapping,
    ComplexMapping,
    Comment,
}

impl YamlParser {
    pub fn parse(source: &str) -> Result<Document, YamlParseError> {
        let mut docs = Vec::new();
        let mut parser = vec![Self::Initial];
        let mut chars = source.chars().n_peekable::<3>();
        loop {
            match parser.last().unwrap() {
                YamlParser::Initial => {
                    let char = chars.next();
                    if char.is_none() {
                        break Ok(Document::Single(Yaml::Scalar(None))); //empty file is single yaml has top-level null
                    }
                    let char = char.unwrap();
                    match char {
                        '"' | '\'' => parser.push(YamlParser::StringScalar { quote: char }),
                    };
                }

                YamlParser::LiteralScalar => todo!(),
                YamlParser::FoldedScalar => todo!(),
                YamlParser::Sequence => todo!(),
                YamlParser::Mapping => todo!(),
                YamlParser::ComplexMapping => todo!(),
                YamlParser::Comment => todo!(),
                YamlParser::StringScalar { quote } => todo!(),
                YamlParser::AutoScalar => todo!(),
            }
        }
    }
}

pub enum YamlParseError {}
