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

    fn ask_name(&self, parent_id: &str, child: &serde_json::Value) -> String {
        match self {
            Parsers::TextFileUTF8 => text_file_utf8::ask_name(parent_id, child),
        }
    }

    fn ask_explain(&self, id: &str, raw_bytes: &[u8], data: &serde_json::Value) -> String {
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
            Ok(result) => serde_json::Value::Array(result),
            Err(_) => parse_failure(&data, 0),
        },
    )
}

struct ParseResult {
    body: Result<Vec<serde_json::Value>, ()>,
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
    let behavior = def.get("type").and_then(|t| t.as_str());

    match behavior {
        Some("repeat0") => parse_repeat0(def, data, min_index, parser),
        Some("until_byte") => parse_until_byte(def, data, min_index, parser),
        _ => ParseResult {
            body: Ok(vec![parse_failure(data, min_index)]),
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
        if let Ok(values) = result.body {
            for value in values {
                children.push(value);
                mut_min_index = result.new_min_index;
            }
        } else {
            break;
        }
    }

    match id_def {
        Some(id) => {
            let mut children_with_name = vec![];
            for child in &children {
                children_with_name.push(serde_json::json!({
                  "name": parser.ask_name(id, &child),
                  "data": child
                }));
            }
            let mut body = serde_json::json!({"minIndex": min_index, "maxIndex": mut_min_index, "children":children_with_name});

            let explain = parser.ask_explain(id, &data[min_index..mut_min_index], &body);

            body.as_object_mut()
                .unwrap()
                .insert("explain".to_owned(), serde_json::Value::String(explain));

            ParseResult {
                body: Ok(vec![body]),
                new_min_index: mut_min_index,
            }
        }
        None => ParseResult {
            body: Ok(children),
            new_min_index: mut_min_index,
        },
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
        let explain = parser.ask_explain(id, &data[min_index..new_min_index], &body);

        body.as_object_mut()
            .unwrap()
            .insert("explain".to_owned(), serde_json::Value::String(explain));
    }

    ParseResult {
        body: Ok(vec![body]),
        new_min_index,
    }
}
