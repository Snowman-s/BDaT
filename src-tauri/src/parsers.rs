mod text_file_utf8;

use ::phf::{phf_map, Map};

#[derive(Clone, Copy)]
pub enum Parsers {
    TextFileUTF8,
}

impl Parsers {
    fn get_syntax(&self) -> &'static [u8] {
        match self {
            Parsers::TextFileUTF8 => include_bytes!("./parsers/text_file_utf8.yaml"),
        }
    }

    fn ask_name(&self, parent_id: &str, child: &serde_json::Value) -> Option<String> {
        match self {
            Parsers::TextFileUTF8 => text_file_utf8::ask_name(parent_id, child),
        }
    }

    fn ask_explain(&self, id: &str, raw_bytes: &[u8], data: &serde_json::Value) -> Option<String> {
        match self {
            Parsers::TextFileUTF8 => text_file_utf8::ask_explain(id, raw_bytes, &data),
        }
    }
}

pub static PARSERS_LIST: Map<&'static str, Parsers> = phf_map! {
"Text File(UTF-8)" => Parsers::TextFileUTF8,
};

pub fn inner_parse(parser: &str, data: Vec<u8>) -> serde_json::Value {
    PARSERS_LIST.get(parser).map_or_else(
        || parse_failure(&data, 0),
        |parser| match parse(parser, &data).body {
            Ok(result) => result,
            Err(_) => parse_failure(&data, 0),
        },
    )
}

struct ParseResult {
    body: Result<serde_json::Value, ()>,
    new_min_index: usize,
}

fn parse_failure(data: &Vec<u8>, min_index: usize) -> serde_json::Value {
    serde_json::json!({
      "explain": "解析失敗",
      "range": {
        "minIndex": min_index,
        "maxIndex": data.len()
      },
      "children":[]
    })
}

fn parse(parser: &Parsers, data: &Vec<u8>) -> ParseResult {
    let binding = serde_yaml::from_slice::<serde_yaml::Value>(parser.get_syntax()).unwrap();
    let def = binding.as_mapping().unwrap();

    parse_main(def, data, 0, parser)
}

fn parse_main(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    parser: &Parsers,
) -> ParseResult {
    let id = def.get("id").and_then(|t| t.as_str());
    let behavior = def.get("type").and_then(|t| t.as_str());

    match behavior {
        Some("repeat0") => parse_repeat0(def, data, min_index, parser),
        Some("until_byte") => parse_until_byte(def, data, min_index, parser),
        default => ParseResult {
            body: Ok(parse_failure(data, min_index)),
            new_min_index: data.len(),
        },
    }
}

fn parse_repeat0(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    parser: &Parsers,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let child_def = def.get("children").unwrap().as_mapping().unwrap();

    let mut children: Vec<serde_json::Value> = vec![];

    let mut mut_min_index = min_index;

    loop {
        if data.len() <= mut_min_index {
            break;
        }

        let result = parse_main(child_def, data, mut_min_index, parser);
        if let Ok(value) = result.body {
            if let Some(id) = id_def {
                let opt_name = parser.ask_name(id, &value);

                if let Some(name) = opt_name {
                    children.push(serde_json::json!(
                      {
                        "name": name,
                        "data": value
                      }
                    ));
                }
            }
            mut_min_index = result.new_min_index;
        } else {
            break;
        }
    }

    let mut body =
        serde_json::json!({"minIndex": min_index, "maxIndex": mut_min_index, "children":children});

    if let Some(id) = id_def {
        let opt_explain = parser.ask_explain(id, &data[min_index..mut_min_index], &body);

        if let Some(explain) = opt_explain {
            body.as_object_mut()
                .unwrap()
                .insert("explain".to_owned(), serde_json::Value::String(explain));
        }
    }

    ParseResult {
        body: Ok(body),
        new_min_index: mut_min_index,
    }
}

fn parse_until_byte(
    def: &serde_yaml::Mapping,
    data: &Vec<u8>,
    min_index: usize,
    parser: &Parsers,
) -> ParseResult {
    let id_def = def.get("id").and_then(|v| v.as_str());
    let byte_def: u8 = def
        .get("byte")
        .and_then(|v| v.as_u64())
        .unwrap()
        .try_into()
        .unwrap();

    let mut new_min_index = data.len();
    for i in min_index..data.len() {
        if data[i] == byte_def {
            new_min_index = i + 1;
            break;
        }
    }

    let mut body =
        serde_json::json!({"minIndex": min_index, "maxIndex": new_min_index, "children":[]});

    if let Some(id) = id_def {
        let opt_explain = parser.ask_explain(id, &data[min_index..new_min_index], &body);

        if let Some(explain) = opt_explain {
            body.as_object_mut()
                .unwrap()
                .insert("explain".to_owned(), serde_json::Value::String(explain));
        }
    }

    ParseResult {
        body: Ok(body),
        new_min_index,
    }
}
