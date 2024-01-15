mod text_file_utf8;

use ::phf::{phf_map, Map};
use text_file_utf8::parse_text_file_utf8;

#[derive(Clone, Copy)]
pub enum Parsers {
    TextFileUTF8,
}

pub static PARSERS_LIST: Map<&'static str, Parsers> = phf_map! {
  "Text File(UTF-8)" => Parsers::TextFileUTF8,
};

pub fn inner_parse(parser: &str, data: Vec<u8>) -> serde_json::Value {
    PARSERS_LIST
        .get(parser)
        .cloned()
        .map_or_else(
            || parse_failure(&data),
            |parser| match parser {
                Parsers::TextFileUTF8 => parse_text_file_utf8(&data),
            },
        )
        .into()
}

fn parse_failure(data: &Vec<u8>) -> serde_json::Value {
    serde_json::json!({
      "explain": "解析失敗",
      "range": {
        "minIndex": 0,
        "mqxIndex": data.len()
      },
      "children":[]
    })
}
